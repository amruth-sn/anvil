use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use anyhow::Result;
use colored::*;
use serde_yaml;

use anvil_engine::{
    TemplateConfig, TemplateEngine, Context, FileGenerator,
    CompositionEngine, ServiceSelection, ServiceCategory
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

        // Service selection flags
        #[arg(long)]
        auth: Option<String>,
        
        #[arg(long)]
        payments: Option<String>,
        
        #[arg(long)]
        database: Option<String>,
        
        #[arg(long)]
        ai: Option<String>,
        
        #[arg(long)]
        deployment: Option<String>,
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
            dry_run,
            auth,
            payments,
            database,
            ai,
            deployment,
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
                auth,
                payments,
                database,
                ai,
                deployment,
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
    // Service selections
    auth: Option<String>,
    payments: Option<String>,
    database: Option<String>,
    ai: Option<String>,
    deployment: Option<String>,
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
    
    // Check if services are specified to use composition
    let services = collect_service_selections(&options)?;
    
    let processed_template = if !services.is_empty() {
        println!("{} Using template composition with {} services...", "⚙️".bright_blue(), services.len());
        
        let templates_dir = find_templates_directory()?;
        let shared_dir = templates_dir.join("shared");
        
        let composition_engine = CompositionEngine::new(templates_dir, shared_dir);
        
        if options.verbose {
            println!("{} Composing template with {} services...", "🔧".bright_blue(), services.len());
        }
        
        let composed = composition_engine.compose_template(&template_name, services).await
            .map_err(|e| anyhow::anyhow!("Template composition failed: {}", e))?;
        
        if options.verbose {
            println!("{} Composition complete, processing {} files...", "✅".bright_green(), composed.files.len());
        }
        
        // Convert ComposedTemplate to ProcessedTemplate
        engine.process_composed_template(composed, &context).await
            .map_err(|e| anyhow::anyhow!("Template processing failed: {}", e))?
    } else {
        println!("{} Processing template files...", "⚙️".bright_blue());
        engine.process_template(&template_dir, &context).await
            .map_err(|e| anyhow::anyhow!("Template processing failed: {}", e))?
    };
    
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

async fn build_context(config: &TemplateConfig, options: &CreateOptions) -> Result<Context> {
    let mut context_builder = Context::builder()
        .variable("project_name", options.name.clone());
    
    // Add variables based on template configuration with default values
    for variable in &config.variables {
        // Don't override project_name if it's already set
        if variable.name != "project_name" {
            let value = match variable.default.as_ref() {
                Some(default) => default.clone(),
                None => {
                    // Provide sensible defaults for template variables
                    match variable.name.as_str() {
                        "project_description" => serde_yaml::Value::String("A modern SaaS application".to_string()),
                        "author_name" => serde_yaml::Value::String("".to_string()),
                        "domain" => serde_yaml::Value::String("myapp.com".to_string()),
                        _ => serde_yaml::Value::String("".to_string()),
                    }
                }
            };
            context_builder = context_builder.variable(variable.name.clone(), value);
        }
    }
    
    let mut context = context_builder.build();
    
    // Add available features from template
    for feature in &config.features {
        context.add_feature(feature.name.clone());
    }
    
    // Add default services object for templates that expect it
    context.add_variable("services".to_string(), serde_yaml::Value::Mapping({
        let mut services = serde_yaml::Mapping::new();
        services.insert(
            serde_yaml::Value::String("auth".to_string()),
            serde_yaml::Value::String("none".to_string())
        );
        services.insert(
            serde_yaml::Value::String("payments".to_string()),
            serde_yaml::Value::String("none".to_string())
        );
        services.insert(
            serde_yaml::Value::String("database".to_string()),
            serde_yaml::Value::String("none".to_string())
        );
        services.insert(
            serde_yaml::Value::String("ai".to_string()),
            serde_yaml::Value::String("none".to_string())
        );
        services.insert(
            serde_yaml::Value::String("deployment".to_string()),
            serde_yaml::Value::String("none".to_string())
        );
        services
    }));
    
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

fn find_templates_directory() -> Result<PathBuf> {
    let templates_dir = std::env::current_dir()?.join("templates");
    
    if !templates_dir.exists() {
        return Err(anyhow::anyhow!("Templates directory not found: {}", templates_dir.display()));
    }
    
    Ok(templates_dir)
}

fn collect_service_selections(options: &CreateOptions) -> Result<Vec<ServiceSelection>> {
    let mut services = Vec::new();
    
    if let Some(auth) = &options.auth {
        services.push(ServiceSelection {
            category: ServiceCategory::Auth,
            provider: auth.clone(),
            config: std::collections::HashMap::new(),
        });
    }
    
    if let Some(payments) = &options.payments {
        services.push(ServiceSelection {
            category: ServiceCategory::Payments,
            provider: payments.clone(),
            config: std::collections::HashMap::new(),
        });
    }
    
    if let Some(database) = &options.database {
        services.push(ServiceSelection {
            category: ServiceCategory::Database,
            provider: database.clone(),
            config: std::collections::HashMap::new(),
        });
    }
    
    if let Some(ai) = &options.ai {
        services.push(ServiceSelection {
            category: ServiceCategory::AI,
            provider: ai.clone(),
            config: std::collections::HashMap::new(),
        });
    }
    
    if let Some(deployment) = &options.deployment {
        services.push(ServiceSelection {
            category: ServiceCategory::Deployment,
            provider: deployment.clone(),
            config: std::collections::HashMap::new(),
        });
    }
    
    Ok(services)
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
