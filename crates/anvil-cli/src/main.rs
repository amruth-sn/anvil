use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use anyhow::Result;
use colored::*;

use anvil_engine::{
    TemplateConfig, TemplateEngine, Context, FileGenerator
};

/// Anvil - Universal template engine for developers
#[derive(Parser)]
#[command(name = "anvil")]
#[command(about = "Universal template engine for developers")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
    
    /// Configuration file path
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new project from template
    Create {
        /// Project name
        name: String,
        
        /// Template to use
        #[arg(short, long)]
        template: Option<String>,
        
        /// Output directory
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
        
        /// Skip interactive prompts
        #[arg(long)]
        no_input: bool,
        
        /// Initialize Git repository
        #[arg(long)]
        git: bool,
        
        /// Create GitHub repository
        #[arg(long)]
        github: bool,
        
        /// Clean output directory if it exists
        #[arg(long)]
        force: bool,
        
        /// Dry run - don't actually create files
        #[arg(long)]
        dry_run: bool,
    },
    
    /// List available templates
    List {
        /// Filter by language
        #[arg(short, long)]
        language: Option<String>,
        
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
    
    /// Search templates
    Search {
        /// Search query
        query: String,
        
        /// Maximum results to show
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
    
    // Set up logging based on verbosity
    let log_level = if cli.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .init();
    
    // Execute command
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

/// Options for the create command
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

/// Create a new project from template
async fn create_project(options: CreateOptions) -> Result<()> {
    println!("{} Creating project '{}'...", "🛠️".bright_blue(), options.name.bright_green());
    
    // Determine template to use
    let template_name = match &options.template {
        Some(name) => name.clone(),
        None => {
            if options.no_input {
                return Err(anyhow::anyhow!("Template must be specified when using --no-input"));
            }
            
            // Interactive template selection (for now, default to rust-hello-world)
            println!("{} No template specified, using default 'rust-hello-world'", "ℹ️".bright_blue());
            "rust-hello-world".to_string()
        }
    };
    
    // Determine output directory
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
    
    // Load template configuration
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
    
    // Build context from variables
    let context = build_context(&template_config, &options).await?;
    
    // Check output directory
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
    
    // Process template
    let mut engine = TemplateEngine::new()
        .map_err(|e| anyhow::anyhow!("Failed to create template engine: {}", e))?;
    
    engine.validate_context(&context, &template_config)
        .map_err(|e| anyhow::anyhow!("Context validation failed: {}", e))?;
    
    println!("{} Processing template files...", "⚙️".bright_blue());
    let processed_template = engine.process_template(&template_dir, &context).await
        .map_err(|e| anyhow::anyhow!("Template processing failed: {}", e))?;
    
    // Generate files with progress reporting
    let progress_callback = if !options.verbose {
        Some(Box::new(|current: usize, total: usize, _msg: &str| {
            print!("\r{} Processing files: {}/{}", "📄".bright_blue(), current, total);
            if current == total {
                println!(); // New line when complete
            }
        }) as Box<dyn Fn(usize, usize, &str) + Send + Sync>)
    } else {
        None
    };
    
    let result = generator.generate_files(processed_template, progress_callback).await
        .map_err(|e| anyhow::anyhow!("File generation failed: {}", e))?;
    
    // Report results
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
    
    // Future: Git initialization, GitHub creation, etc.
    if options.git && !options.dry_run {
        println!("{} Git initialization will be implemented in Stage 3", "🔮".bright_magenta());
    }
    
    if options.github && !options.dry_run {
        println!("{} GitHub integration will be implemented in Stage 4", "🔮".bright_magenta());
    }
    
    Ok(())
}

/// Build context from template configuration and user input
async fn build_context(_config: &TemplateConfig, options: &CreateOptions) -> Result<Context> {
    let context = Context::builder()
        .variable("project_name", options.name.clone())
        .build();
    
    // For now, add basic variables
    // In Stage 2, this will include interactive prompts for all template variables
    
    if options.verbose {
        println!("{} Built context with {} variables", "🎯".bright_blue(), context.variables().len());
    }
    
    Ok(context)
}

/// Find template directory (for now, look in templates/ subdirectory)
fn find_template_directory(template_name: &str) -> Result<PathBuf> {
    let templates_dir = std::env::current_dir()?.join("templates");
    let template_dir = templates_dir.join(template_name);
    
    if !template_dir.exists() {
        return Err(anyhow::anyhow!("Template '{}' not found in {}", template_name, templates_dir.display()));
    }
    
    Ok(template_dir)
}

/// List available templates
async fn list_templates(_language: Option<String>, _format: OutputFormat) -> Result<()> {
    println!("{} Template listing will be implemented in Stage 3", "🔮".bright_magenta());
    
    // For now, just show the rust-hello-world template if it exists
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

/// Search templates
async fn search_templates(_query: String, _limit: usize) -> Result<()> {
    println!("{} Template search will be implemented in Stage 3", "🔮".bright_magenta());
    Ok(())
}
