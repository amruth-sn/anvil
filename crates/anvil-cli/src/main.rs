use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use anyhow::Result;
use colored::*;

use anvil_engine::{
    TemplateConfig, TemplateEngine, Context, FileGenerator
};

#[derive(Parser)]
#[command(name = "anvil")]
#[command(about = "Universal template engine for developers")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    #[arg(short, long, global = true)]
    pub verbose: bool,
    
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    Create {
        name: String,
        
        #[arg(short, long)]
        template: Option<String>,
        
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
        
        #[arg(long)]
        no_input: bool,
        
        #[arg(long)]
        git: bool,
        
        #[arg(long)]
        github: bool,
        
        #[arg(long)]
        force: bool,
        
        #[arg(long)]
        dry_run: bool,
    },
    
    List {
        #[arg(short, long)]
        language: Option<String>,
        
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
    
    Search {
        query: String,
        
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}

#[derive(ValueEnum, Clone)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let log_level = if cli.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .init();
    
    match cli.command {
        Commands::Create { 
            name, 
            template, 
            output, 
            no_input, 
            git, 
            github, 
            force,
            dry_run 
        } => {
            create_project(CreateOptions {
                name,
                template,
                output,
                no_input,
                git,
                github,
                force,
                dry_run,
                verbose: cli.verbose,
            }).await?;
        }
        Commands::List { language, format } => {
            list_templates(language, format).await?;
        }
        Commands::Search { query, limit } => {
            search_templates(query, limit).await?;
        }
    }
    
    Ok(())
}

#[derive(Debug)]
struct CreateOptions {
    name: String,
    template: Option<String>,
    output: PathBuf,
    no_input: bool,
    git: bool,
    github: bool,
    force: bool,
    dry_run: bool,
    verbose: bool,
}

async fn create_project(options: CreateOptions) -> Result<()> {
    println!("{} Creating project '{}'...", "🛠️".bright_blue(), options.name.bright_green());
    
    let template_name = match &options.template {
        Some(name) => name.clone(),
        None => {
            if options.no_input {
                return Err(anyhow::anyhow!("Template must be specified when using --no-input"));
            }
            
            println!("{} No template specified, using default 'rust-hello-world'", "ℹ️".bright_blue());
            "rust-hello-world".to_string()
        }
    };
    
    let output_dir = if options.output == PathBuf::from(".") {
        std::env::current_dir()?.join(&options.name)
    } else {
        options.output.join(&options.name)
    };
    
    if options.verbose {
        println!("{} Template: {}", "📋".bright_blue(), template_name.bright_yellow());
        println!("{} Output directory: {}", "📁".bright_blue(), output_dir.display().to_string().bright_yellow());
        println!("{} Dry run: {}", "🔍".bright_blue(), options.dry_run.to_string().bright_yellow());
    }
    
    let template_dir = find_template_directory(&template_name)?;
    let config_path = template_dir.join("anvil.yaml");
    
    if !config_path.exists() {
        return Err(anyhow::anyhow!("Template configuration not found: {}", config_path.display()));
    }
    
    let template_config = TemplateConfig::from_file(&config_path).await
        .map_err(|e| anyhow::anyhow!("Failed to load template config: {}", e))?;
    
    if options.verbose {
        println!("{} Loaded template: {} v{}", "✅".bright_green(), template_config.name, template_config.version);
        println!("{} Description: {}", "📝".bright_blue(), template_config.description);
    }
    
    let context = build_context(&template_config, &options).await?;
    
    let generator = if options.dry_run {
        FileGenerator::new_dry_run(&output_dir)
    } else {
        FileGenerator::new(&output_dir)
    };
    
    let dir_status = generator.check_output_directory().await
        .map_err(|e| anyhow::anyhow!("Failed to check output directory: {}", e))?;
    
    match dir_status {
        anvil_engine::generator::DirectoryStatus::ExistsWithContent => {
            if !options.force {
                return Err(anyhow::anyhow!(
                    "Output directory '{}' already exists and is not empty. Use --force to overwrite.",
                    output_dir.display()
                ));
            }
            
            if !options.dry_run {
                println!("{} Cleaning existing directory...", "🧹".bright_yellow());
                generator.clean_output_directory().await
                    .map_err(|e| anyhow::anyhow!("Failed to clean output directory: {}", e))?;
            }
        }
        anvil_engine::generator::DirectoryStatus::DoesNotExist => {
            println!("{} Creating new directory...", "📁".bright_blue());
        }
        anvil_engine::generator::DirectoryStatus::ExistsEmpty => {
            if options.verbose {
                println!("{} Using existing empty directory", "📁".bright_blue());
            }
        }
    }
    
    let mut engine = TemplateEngine::new_for_testing()
        .map_err(|e| anyhow::anyhow!("Failed to create template engine: {}", e))?;
    
    engine.validate_context(&context, &template_config)
        .map_err(|e| anyhow::anyhow!("Context validation failed: {}", e))?;
    
    println!("{} Processing template files...", "⚙️".bright_blue());
    let processed_template = engine.process_template(&template_dir, &context).await
        .map_err(|e| anyhow::anyhow!("Template processing failed: {}", e))?;
    
    let progress_callback = if !options.verbose {
        Some(Box::new(|current: usize, total: usize, _msg: &str| {
            print!("\r{} Processing files: {}/{}", "📄".bright_blue(), current, total);
            if current == total {
            }
        }) as Box<dyn Fn(usize, usize, &str) + Send + Sync>)
    } else {
        None
    };
    
    let result = generator.generate_files(processed_template, progress_callback).await
        .map_err(|e| anyhow::anyhow!("File generation failed: {}", e))?;
    
    if options.dry_run {
        println!("{} Dry run completed successfully!", "✅".bright_green());
        println!("  {} {} files would be created", "📄".bright_blue(), result.files_created);
        println!("  {} {} directories would be created", "📁".bright_blue(), result.directories_created);
        println!("  {} {} bytes would be written", "💾".bright_blue(), result.bytes_written);
    } else {
        println!("{} Project created successfully!", "✅".bright_green());
        println!("  {} {} files created", "📄".bright_blue(), result.files_created);
        println!("  {} {} directories created", "📁".bright_blue(), result.directories_created);
        println!("  {} {} bytes written", "💾".bright_blue(), result.bytes_written);
        println!("  {} Location: {}", "📍".bright_blue(), result.output_directory.display().to_string().bright_yellow());
    }
    
    if options.git && !options.dry_run {
        println!("{} Git initialization will be implemented in Stage 3", "🔮".bright_magenta());
    }
    
    if options.github && !options.dry_run {
        println!("{} GitHub integration will be implemented in Stage 4", "🔮".bright_magenta());
    }
    
    Ok(())
}

async fn build_context(_config: &TemplateConfig, options: &CreateOptions) -> Result<Context> {
    let mut context = Context::builder()
        .variable("project_name", options.name.clone())
        .variable("author_name", "Test Author".to_string())
        .variable("description", "A test Rust project".to_string())
        .build();
    
    context.add_feature("cli".to_string());
    context.add_feature("tests".to_string());
    
    if options.verbose {
        println!("{} Built context with {} variables and {} features", 
                "🎯".bright_blue(), 
                context.variables().len(),
                context.features().len());
    }
    
    Ok(context)
}

fn find_template_directory(template_name: &str) -> Result<PathBuf> {
    let templates_dir = std::env::current_dir()?.join("templates");
    let template_dir = templates_dir.join(template_name);
    
    if !template_dir.exists() {
        return Err(anyhow::anyhow!("Template '{}' not found in {}", template_name, templates_dir.display()));
    }
    
    Ok(template_dir)
}

async fn list_templates(_language: Option<String>, _format: OutputFormat) -> Result<()> {
    println!("{} Template listing will be implemented in Stage 3", "🔮".bright_magenta());
    
    let templates_dir = std::env::current_dir()?.join("templates");
    if templates_dir.exists() {
        println!("{} Available templates:", "📋".bright_blue());
        
        for entry in std::fs::read_dir(&templates_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let template_name = entry.file_name().to_string_lossy().to_string();
                let config_path = entry.path().join("anvil.yaml");
                
                if config_path.exists() {
                    match TemplateConfig::from_file(&config_path).await {
                        Ok(config) => {
                            println!("  {} {} - {}", "•".bright_green(), template_name.bright_yellow(), config.description);
                        }
                        Err(_) => {
                            println!("  {} {} - {}", "•".bright_red(), template_name.bright_yellow(), "Invalid configuration");
                        }
                    }
                } else {
                    println!("  {} {} - {}", "•".bright_red(), template_name.bright_yellow(), "No configuration file");
                }
            }
        }
    } else {
        println!("{} No templates directory found. Create 'templates/' directory with your templates.", "⚠️".bright_yellow());
    }
    
    Ok(())
}

async fn search_templates(_query: String, _limit: usize) -> Result<()> {
    println!("{} Template search will be implemented in Stage 3", "🔮".bright_magenta());
    Ok(())
}
