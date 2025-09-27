/*
Module for template composition engine that combines base templates with service components.
Handles file merging, conflict resolution, and conditional inclusion logic.
*/

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde_json::Value;
use serde::Serialize;
use tokio::fs;

use crate::config::{TemplateConfig, ServiceCategory, CompositionConfig, FileMergingStrategy};
use crate::error::{EngineError, EngineResult};

#[derive(Debug, Clone)]
pub struct CompositionEngine {
    base_template_path: PathBuf,
    shared_services_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ServiceSelection {
    pub category: ServiceCategory,
    pub provider: String,
    pub config: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct ComposedTemplate {
    pub base_config: TemplateConfig,
    pub files: Vec<ComposedFile>,
    pub merged_dependencies: HashMap<String, Value>,
    pub environment_variables: HashMap<String, String>,
    pub service_context: ServiceContext,
}

#[derive(Debug, Clone)]
pub struct ComposedFile {
    pub path: PathBuf,
    pub content: String,
    pub source: FileSource,
    pub merge_strategy: FileMergingStrategy,
    pub is_template: bool,
}

#[derive(Debug, Clone)]
pub enum FileSource {
    BaseTemplate,
    Service { category: ServiceCategory, provider: String },
    Merged,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceContext {
    pub services: HashMap<String, ServiceInfo>,
    pub shared_config: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceInfo {
    pub provider: String,
    pub config: HashMap<String, Value>,
    pub exports: HashMap<String, Value>,
}

impl CompositionEngine {
    pub fn new(base_template_path: PathBuf, shared_services_path: PathBuf) -> Self {
        Self {
            base_template_path,
            shared_services_path,
        }
    }
    
    pub fn shared_services_path(&self) -> &PathBuf {
        &self.shared_services_path
    }
    
    /*
    Discovers available service providers for a given category by scanning
    the shared services directory structure.
    */
    pub async fn discover_service_providers(&self, category: ServiceCategory) -> EngineResult<Vec<String>> {
        let category_dir = self.shared_services_path
            .join(format!("{:?}", category).to_lowercase());
        
        if !category_dir.exists() {
            return Ok(vec!["none".to_string()]);
        }
        
        let mut providers = vec!["none".to_string()];
        let mut entries = fs::read_dir(&category_dir).await
            .map_err(|e| EngineError::file_error(&category_dir, e))?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| EngineError::file_error(&category_dir, e))? {
            
            let path = entry.path();
            if path.is_dir() {
                if let Some(provider_name) = path.file_name().and_then(|n| n.to_str()) {
                    // Check if provider has a valid anvil.yaml config
                    let config_path = path.join("anvil.yaml");
                    if config_path.exists() {
                        providers.push(provider_name.to_string());
                    }
                }
            }
        }
        
        providers.sort();
        Ok(providers)
    }
    
    /*
    Discovers all available services and their providers by scanning the
    shared services directory structure.
    */
    pub async fn discover_all_services(&self) -> EngineResult<HashMap<ServiceCategory, Vec<String>>> {
        let mut all_services = HashMap::new();
        
        // Scan each service category
        for category in [
            ServiceCategory::Auth,
            ServiceCategory::Payments, 
            ServiceCategory::Database,
            ServiceCategory::AI,
            ServiceCategory::Api,
            ServiceCategory::Deployment,
            ServiceCategory::Monitoring,
            ServiceCategory::Email,
            ServiceCategory::Storage,
        ] {
            let providers = self.discover_service_providers(category.clone()).await?;
            all_services.insert(category, providers);
        }
        
        Ok(all_services)
    }

    /*
    Composes a base template with selected services.
    Returns a ComposedTemplate containing all files and configurations.
    */
    pub async fn compose_template(
        &self,
        template_name: &str,
        services: Vec<ServiceSelection>,
    ) -> EngineResult<ComposedTemplate> {
        // Load base template configuration
        let base_config_path = self.base_template_path.join(template_name).join("anvil.yaml");
        let base_config = TemplateConfig::from_file(&base_config_path).await?;

        // Validate service selections
        self.validate_service_selections(&base_config, &services).await?;

        // Build service dependency injection context
        let service_context = self.build_service_context(&services).await?;

        // Collect all files from base template
        let mut composed_files = self.collect_base_template_files(template_name).await?;

        // Add service-specific files
        for service in &services {
            let service_files = self.collect_service_files(&service).await?;
            composed_files.extend(service_files);
        }

        // Apply conditional file inclusion
        let filtered_files = self.apply_conditional_inclusion(composed_files, &services, &base_config.composition).await?;

        // Handle file conflicts and merging
        let resolved_files = self.resolve_file_conflicts(filtered_files, &base_config.composition).await?;

        // Merge dependencies (package.json, Cargo.toml, etc.)
        let merged_dependencies = self.merge_dependencies(&services).await?;

        // Collect environment variables
        let environment_variables = self.collect_environment_variables(&services).await?;

        Ok(ComposedTemplate {
            base_config,
            files: resolved_files,
            merged_dependencies,
            environment_variables,
            service_context,
        })
    }

    /*
    Validates that selected services are compatible with the base template
    and with each other. Checks dependencies and conflicts.
    */
    async fn validate_service_selections(
        &self,
        base_config: &TemplateConfig,
        services: &[ServiceSelection],
    ) -> EngineResult<()> {
        // Check that required services are provided
        for service_def in &base_config.services {
            if service_def.required {
                let has_service = services.iter().any(|s| s.category == service_def.category);
                if !has_service {
                    return Err(EngineError::composition_error(format!(
                        "Required service '{}' not provided", service_def.name
                    )));
                }
            }
        }

        // Validate each service selection
        for service in services {
            // Check if service category is supported by the template
            let service_def = base_config.services.iter()
                .find(|s| s.category == service.category)
                .ok_or_else(|| EngineError::composition_error(format!(
                    "Service category '{:?}' not supported by template '{}'", 
                    service.category, base_config.name
                )))?;

            // Check if provider option is valid
            if !service_def.options.contains(&service.provider) {
                return Err(EngineError::composition_error(format!(
                    "Invalid provider '{}' for service '{:?}'. Valid options: {:?}",
                    service.provider, service.category, service_def.options
                )));
            }

            // Check if service files exist
            let service_path = self.shared_services_path
                .join(format!("{:?}", service.category).to_lowercase())
                .join(&service.provider);
            
            if !service_path.exists() {
                return Err(EngineError::composition_error(format!(
                    "Service files not found for '{:?}/{}'", 
                    service.category, service.provider
                )));
            }
        }

        // Check for conflicting services
        for service in services {
            if let Some(service_def) = base_config.services.iter().find(|s| s.category == service.category) {
                if let Some(conflicts) = &service_def.conflicts {
                    for conflict in conflicts {
                        if services.iter().any(|s| format!("{:?}", s.category).to_lowercase() == *conflict) {
                            return Err(EngineError::composition_error(format!(
                                "Service conflict: {} conflicts with {}", 
                                service_def.name, conflict
                            )));
                        }
                    }
                }
            }
        }

        // Check for missing dependencies
        for service in services {
            if let Some(service_def) = base_config.services.iter().find(|s| s.category == service.category) {
                if let Some(dependencies) = &service_def.dependencies {
                    for dependency in dependencies {
                        let has_dependency = services.iter().any(|s| 
                            format!("{:?}", s.category).to_lowercase() == *dependency
                        );
                        
                        if !has_dependency {
                            return Err(EngineError::composition_error(format!(
                                "Service '{}' requires dependency '{}' which is not selected",
                                service_def.name, dependency
                            )));
                        }
                    }
                }
            }
        }

        // Enhanced compatibility validation
        self.validate_service_compatibility(base_config, services).await?;

        Ok(())
    }

    /*
    Enhanced compatibility validation that checks language requirements,
    compatibility rules, and cross-service dependencies.
    */
    async fn validate_service_compatibility(
        &self,
        base_config: &TemplateConfig,
        services: &[ServiceSelection],
    ) -> EngineResult<()> {
        // Detect project language from base template
        let project_language = self.detect_project_language(base_config).await?;
        
        // Load service configurations for enhanced validation
        for service in services {
            let service_config_path = self.shared_services_path
                .join(format!("{:?}", service.category).to_lowercase())
                .join(&service.provider)
                .join("anvil.yaml");

            if service_config_path.exists() {
                let service_config = crate::config::ServiceConfig::from_file(&service_config_path).await?;
                
                // Check language requirements
                if let Some(language_reqs) = self.get_service_language_requirements(&service_config).await? {
                    for required_lang in &language_reqs {
                        if !project_language.contains(required_lang) {
                            return Err(EngineError::composition_error(format!(
                                "Service '{}/{}' requires {} but project language is {:?}",
                                format!("{:?}", service.category).to_lowercase(),
                                service.provider,
                                required_lang,
                                project_language
                            )));
                        }
                    }
                }
                
                // Check compatibility rules from service configuration
                self.validate_service_rules(&service_config, services, &project_language).await?;
            }
        }

        // Check cross-service compatibility
        self.validate_cross_service_compatibility(services).await?;

        Ok(())
    }

    /*
    Detects the primary language(s) of the project from the base template.
    */
    async fn detect_project_language(&self, base_config: &TemplateConfig) -> EngineResult<Vec<String>> {
        let mut languages = Vec::new();
        
        // Check template name for language hints
        if base_config.name.contains("rust") {
            languages.push("rust".to_string());
        } else if base_config.name.contains("go") {
            languages.push("go".to_string());
        } else if base_config.name.contains("python") {
            languages.push("python".to_string());
        } else {
            // Default to TypeScript for fullstack templates
            languages.push("typescript".to_string());
            languages.push("javascript".to_string());
        }
        
        // TODO: Could be enhanced to check actual files in template
        // for more accurate language detection
        
        Ok(languages)
    }
    
    /*
    Extracts language requirements from service configuration.
    */
    async fn get_service_language_requirements(
        &self,
        service_config: &crate::config::ServiceConfig,
    ) -> EngineResult<Option<Vec<String>>> {
        // For now, we'll check if the service config has language_requirements field
        // This would need to be added to ServiceConfig struct
        // TODO: Add language_requirements field to ServiceConfig
        
        // Hardcoded rules for known services that require specific languages
        match service_config.name.as_str() {
            "trpc-api" => Ok(Some(vec!["typescript".to_string()])),
            _ => Ok(None),
        }
    }
    
    /*
    Validates service-specific compatibility rules.
    */
    async fn validate_service_rules(
        &self,
        service_config: &crate::config::ServiceConfig,
        _services: &[ServiceSelection],
        project_languages: &[String],
    ) -> EngineResult<()> {
        // TODO: Implement compatibility rules validation
        // This would check the compatibility_rules field in service config
        // For now, implement specific known rules
        
        match service_config.name.as_str() {
            "trpc-api" => {
                if !project_languages.contains(&"typescript".to_string()) {
                    return Err(EngineError::composition_error(
                        "tRPC requires TypeScript for type safety".to_string()
                    ));
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /*
    Validates compatibility between different selected services.
    */
    async fn validate_cross_service_compatibility(
        &self,
        services: &[ServiceSelection],
    ) -> EngineResult<()> {
        // Check for known incompatible service combinations
        let _service_providers: std::collections::HashSet<String> = services
            .iter()
            .map(|s| format!("{:?}/{}", s.category, s.provider))
            .collect();
        
        // Example: Check if multiple auth providers are selected
        let auth_services: Vec<_> = services
            .iter()
            .filter(|s| s.category == ServiceCategory::Auth)
            .collect();
        
        if auth_services.len() > 1 {
            let auth_providers: Vec<_> = auth_services
                .iter()
                .map(|s| s.provider.as_str())
                .collect();
            return Err(EngineError::composition_error(format!(
                "Multiple auth providers selected: {:?}. Only one auth provider is allowed.",
                auth_providers
            )));
        }
        
        // Example: Check if multiple API patterns are selected
        let api_services: Vec<_> = services
            .iter()
            .filter(|s| s.category == ServiceCategory::Api)
            .collect();
        
        if api_services.len() > 1 {
            let api_providers: Vec<_> = api_services
                .iter()
                .map(|s| s.provider.as_str())
                .collect();
            return Err(EngineError::composition_error(format!(
                "Multiple API patterns selected: {:?}. Only one API pattern is recommended.",
                api_providers
            )));
        }
        
        // TODO: Add more cross-service compatibility checks
        
        Ok(())
    }

    /*
    Builds service dependency injection context by loading service configurations
    and creating a shared context for cross-service communication.
    */
    async fn build_service_context(&self, services: &[ServiceSelection]) -> EngineResult<ServiceContext> {
        let mut service_context = ServiceContext {
            services: HashMap::new(),
            shared_config: HashMap::new(),
        };

        // Load each service configuration and build context
        for service in services {
            let service_config_path = self.shared_services_path
                .join(format!("{:?}", service.category).to_lowercase())
                .join(&service.provider)
                .join("anvil.yaml");

            if service_config_path.exists() {
                let service_config = crate::config::ServiceConfig::from_file(&service_config_path).await?;
                
                // Build service exports for dependency injection
                let mut exports = HashMap::new();
                
                // Export common service information
                exports.insert("provider".to_string(), Value::String(service.provider.clone()));
                exports.insert("category".to_string(), Value::String(format!("{:?}", service.category)));
                
                // Export service-specific values based on category
                match service.category {
                    ServiceCategory::Auth => {
                        exports.insert("auth_provider".to_string(), Value::String(service.provider.clone()));
                        exports.insert("has_auth".to_string(), Value::Bool(true));
                        // Export auth-specific environment variables
                        for env_var in &service_config.environment_variables {
                            if env_var.name.contains("PUBLISHABLE") || env_var.name.contains("PUBLIC") {
                                exports.insert(
                                    "public_auth_key_name".to_string(), 
                                    Value::String(env_var.name.clone())
                                );
                            }
                        }
                    }
                    ServiceCategory::Database => {
                        exports.insert("database_provider".to_string(), Value::String(service.provider.clone()));
                        exports.insert("has_database".to_string(), Value::Bool(true));
                    }
                    ServiceCategory::Payments => {
                        exports.insert("payments_provider".to_string(), Value::String(service.provider.clone()));
                        exports.insert("has_payments".to_string(), Value::Bool(true));
                    }
                    ServiceCategory::AI => {
                        exports.insert("ai_provider".to_string(), Value::String(service.provider.clone()));
                        exports.insert("has_ai".to_string(), Value::Bool(true));
                    }
                    ServiceCategory::Api => {
                        exports.insert("api_pattern".to_string(), Value::String(service.provider.clone()));
                        exports.insert("has_api".to_string(), Value::Bool(true));
                        exports.insert("api_type".to_string(), Value::String(service.provider.clone()));
                    }
                    _ => {
                        // Generic exports for other service types
                        exports.insert(
                            format!("has_{}", format!("{:?}", service.category).to_lowercase()),
                            Value::Bool(true)
                        );
                    }
                }

                // Merge service-provided exports with user configuration
                let mut all_exports = exports;
                for (key, value) in &service.config {
                    all_exports.insert(format!("config_{}", key), value.clone());
                }

                let service_info = ServiceInfo {
                    provider: service.provider.clone(),
                    config: service.config.clone(),
                    exports: all_exports,
                };

                service_context.services.insert(format!("{:?}", service.category), service_info);
            }
        }

        // Build shared configuration across all services
        let mut has_any_auth = false;
        let mut has_any_database = false;
        
        for (category_str, _service_info) in &service_context.services {
            match category_str.as_str() {
                "Auth" => has_any_auth = true,
                "Database" => has_any_database = true,
                _ => {}
            }
        }

        service_context.shared_config.insert("has_any_auth".to_string(), Value::Bool(has_any_auth));
        service_context.shared_config.insert("has_any_database".to_string(), Value::Bool(has_any_database));
        service_context.shared_config.insert("service_count".to_string(), Value::Number(
            serde_json::Number::from(services.len())
        ));

        Ok(service_context)
    }

    /*
    Collects all files from the base template directory.
    */
    async fn collect_base_template_files(&self, template_name: &str) -> EngineResult<Vec<ComposedFile>> {
        let template_path = self.base_template_path.join(template_name);
        let mut files = Vec::new();

        self.collect_files_recursive(&template_path, &template_path, FileSource::BaseTemplate, &mut files).await?;

        Ok(files)
    }

    /*
    Collects files for a specific service selection.
    */
    async fn collect_service_files(&self, service: &ServiceSelection) -> EngineResult<Vec<ComposedFile>> {
        let service_path = self.shared_services_path
            .join(format!("{:?}", service.category).to_lowercase())
            .join(&service.provider);

        if !service_path.exists() {
            return Err(EngineError::composition_error(format!(
                "Service files not found: {:?}/{}", service.category, service.provider
            )));
        }

        let mut files = Vec::new();
        let source = FileSource::Service {
            category: service.category.clone(),
            provider: service.provider.clone(),
        };

        self.collect_files_recursive(&service_path, &service_path, source, &mut files).await?;

        Ok(files)
    }

    /*
    Recursively collects files from a directory, preserving relative paths.
    */
    fn collect_files_recursive<'a>(
        &'a self,
        dir: &'a Path,
        base_path: &'a Path,
        source: FileSource,
        files: &'a mut Vec<ComposedFile>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = EngineResult<()>> + Send + 'a>> {
        Box::pin(async move {
        let mut entries = fs::read_dir(dir).await.map_err(|e| {
            EngineError::file_error(dir, e)
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            EngineError::file_error(dir, e)
        })? {
            let path = entry.path();
            
            if path.is_dir() {
                self.collect_files_recursive(&path, base_path, source.clone(), files).await?;
            } else if path.is_file() {
                // Skip anvil.yaml config files in service directories
                if path.file_name().and_then(|name| name.to_str()) == Some("anvil.yaml") {
                    continue;
                }

                let relative_path = path.strip_prefix(base_path).map_err(|_| {
                    EngineError::composition_error(format!("Invalid path structure: {}", path.display()))
                })?;

                let content = fs::read_to_string(&path).await.map_err(|e| {
                    EngineError::file_error(&path, e)
                })?;

                // Check if this is a Tera template file
                let is_template = relative_path.extension().and_then(|e| e.to_str()) == Some("tera");
                
                // Process output path - remove .tera extension if present
                let output_path = if is_template {
                    // Remove .tera extension: package.json.tera -> package.json
                    let file_name = relative_path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .trim_end_matches(".tera");
                    relative_path.with_file_name(file_name)
                } else {
                    relative_path.to_path_buf()
                };

                files.push(ComposedFile {
                    path: output_path,
                    content,
                    source: source.clone(),
                    merge_strategy: FileMergingStrategy::default(),
                    is_template,
                });
            }
        }

        Ok(())
        })
    }

    /*
    Resolves file conflicts when multiple sources provide the same file.
    Applies merging strategies based on composition configuration.
    */
    async fn resolve_file_conflicts(
        &self,
        files: Vec<ComposedFile>,
        composition_config: &Option<CompositionConfig>,
    ) -> EngineResult<Vec<ComposedFile>> {
        let mut file_map: HashMap<PathBuf, Vec<ComposedFile>> = HashMap::new();

        // Group files by path
        for file in files {
            file_map.entry(file.path.clone()).or_insert_with(Vec::new).push(file);
        }

        let mut resolved_files = Vec::new();

        for (path, conflicting_files) in file_map {
            if conflicting_files.len() == 1 {
                // No conflict, take the single file
                resolved_files.push(conflicting_files.into_iter().next().unwrap());
            } else {
                // Handle conflict based on strategy
                let default_strategy = FileMergingStrategy::default();
                let strategy = composition_config
                    .as_ref()
                    .map(|c| &c.file_merging_strategy)
                    .unwrap_or(&default_strategy);

                let resolved = self.resolve_single_conflict(path, conflicting_files, strategy).await?;
                resolved_files.push(resolved);
            }
        }

        Ok(resolved_files)
    }

    /*
    Resolves a single file conflict using the specified merging strategy.
    */
    async fn resolve_single_conflict(
        &self,
        path: PathBuf,
        mut files: Vec<ComposedFile>,
        strategy: &FileMergingStrategy,
    ) -> EngineResult<ComposedFile> {
        match strategy {
            FileMergingStrategy::Override => {
                // Service files override base template files
                files.sort_by(|a, b| {
                    match (&a.source, &b.source) {
                        (FileSource::BaseTemplate, FileSource::Service { .. }) => std::cmp::Ordering::Less,
                        (FileSource::Service { .. }, FileSource::BaseTemplate) => std::cmp::Ordering::Greater,
                        _ => std::cmp::Ordering::Equal,
                    }
                });
                Ok(files.into_iter().last().unwrap())
            }
            FileMergingStrategy::Append => {
                // Append all file contents
                let mut combined_content = String::new();
                for file in &files {
                    combined_content.push_str(&file.content);
                    combined_content.push('\n');
                }
                Ok(ComposedFile {
                    path,
                    content: combined_content,
                    source: FileSource::Merged,
                    merge_strategy: FileMergingStrategy::Append,
                    is_template: false,
                })
            }
            FileMergingStrategy::Merge => {
                // Intelligent merging based on file type
                if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                    self.merge_json_files(path, files).await
                } else {
                    // Default to append for non-JSON files
                    let mut combined_content = String::new();
                    for file in &files {
                        combined_content.push_str(&file.content);
                        combined_content.push('\n');
                    }
                    Ok(ComposedFile {
                        path,
                        content: combined_content,
                        source: FileSource::Merged,
                        merge_strategy: FileMergingStrategy::Append,
                        is_template: false,
                    })
                }
            }
            FileMergingStrategy::Skip => {
                // Skip conflicting files, take the first one
                Ok(files.into_iter().next().unwrap())
            }
        }
    }

    /*
    Merges JSON files by combining their objects.
    Handles package.json dependency merging specifically.
    */
    async fn merge_json_files(
        &self,
        path: PathBuf,
        files: Vec<ComposedFile>,
    ) -> EngineResult<ComposedFile> {
        let mut merged_json = serde_json::Map::new();

        for file in &files {
            let json: serde_json::Value = serde_json::from_str(&file.content)
                .map_err(|e| EngineError::composition_error(format!("Invalid JSON in {}: {}", path.display(), e)))?;

            if let serde_json::Value::Object(obj) = json {
                for (key, value) in obj {
                    match merged_json.get(&key) {
                        Some(existing) => {
                            // Merge dependencies and devDependencies
                            if (key == "dependencies" || key == "devDependencies") &&
                               existing.is_object() && value.is_object() {
                                let mut merged_deps = existing.as_object().unwrap().clone();
                                merged_deps.extend(value.as_object().unwrap().clone());
                                merged_json.insert(key, serde_json::Value::Object(merged_deps));
                            } else {
                                // Override for other fields
                                merged_json.insert(key, value);
                            }
                        }
                        None => {
                            merged_json.insert(key, value);
                        }
                    }
                }
            }
        }

        let merged_content = serde_json::to_string_pretty(&merged_json)
            .map_err(|e| EngineError::composition_error(format!("Failed to serialize merged JSON: {}", e)))?;

        Ok(ComposedFile {
            path,
            content: merged_content,
            source: FileSource::Merged,
            merge_strategy: FileMergingStrategy::Merge,
            is_template: false,
        })
    }

    /*
    Merges dependencies from all selected services.
    Returns a map of dependency files to their merged content.
    */
    async fn merge_dependencies(&self, services: &[ServiceSelection]) -> EngineResult<HashMap<String, Value>> {
        let mut merged_deps = HashMap::new();
        let mut npm_dependencies = Vec::new();
        let mut cargo_dependencies = HashMap::new();
        let mut environment_variables = Vec::new();
        
        // Collect dependencies from each service
        for service in services {
            let service_config_path = self.shared_services_path
                .join(format!("{:?}", service.category).to_lowercase())
                .join(&service.provider)
                .join("anvil.yaml");
            
            if service_config_path.exists() {
                let service_config = crate::config::ServiceConfig::from_file(&service_config_path).await?;
                
                // Collect NPM dependencies
                if let Some(deps) = &service_config.dependencies {
                    if let Some(npm_deps) = &deps.npm {
                        npm_dependencies.extend(npm_deps.iter().cloned());
                    }
                    
                    if let Some(cargo_deps) = &deps.cargo {
                        cargo_dependencies.extend(cargo_deps.iter().map(|(k, v)| (k.clone(), v.clone())));
                    }
                }
                
                // Collect environment variables
                environment_variables.extend(service_config.environment_variables);
            }
        }
        
        // Create merged dependencies structure using serde_json::Value (for Tera compatibility)
        if !npm_dependencies.is_empty() {
            let npm_deps: Vec<Value> = npm_dependencies.into_iter()
                .map(|dep| {
                    // Parse dependency with optional version (e.g., "@clerk/nextjs@^5.0.0")
                    if let Some(_at_pos) = dep.rfind('@') {
                        // Check if this is a scoped package or version specifier
                        if dep.starts_with('@') && dep[1..].contains('@') {
                            // Scoped package with version: @clerk/nextjs@^5.0.0
                            let parts: Vec<&str> = dep.rsplitn(2, '@').collect();
                            if parts.len() == 2 {
                                let mut dep_obj = serde_json::Map::new();
                                dep_obj.insert("name".to_string(), Value::String(parts[1].to_string()));
                                dep_obj.insert("version".to_string(), Value::String(parts[0].to_string()));
                                return Value::Object(dep_obj);
                            }
                        } else if !dep.starts_with('@') {
                            // Non-scoped package with version: lodash@4.17.21
                            let parts: Vec<&str> = dep.splitn(2, '@').collect();
                            if parts.len() == 2 {
                                let mut dep_obj = serde_json::Map::new();
                                dep_obj.insert("name".to_string(), Value::String(parts[0].to_string()));
                                dep_obj.insert("version".to_string(), Value::String(parts[1].to_string()));
                                return Value::Object(dep_obj);
                            }
                        }
                    }
                    
                    // Default: package without version specified
                    let mut dep_obj = serde_json::Map::new();
                    dep_obj.insert("name".to_string(), Value::String(dep));
                    dep_obj.insert("version".to_string(), Value::String("^1.0.0".to_string()));
                    Value::Object(dep_obj)
                })
                .collect();
            
            merged_deps.insert("npm".to_string(), Value::Array(npm_deps));
        }
        
        if !cargo_dependencies.is_empty() {
            let cargo_map: serde_json::Map<String, Value> = cargo_dependencies.into_iter()
                .map(|(k, v)| (k, Value::String(v)))
                .collect();
            merged_deps.insert("cargo".to_string(), Value::Object(cargo_map));
        }
        
        if !environment_variables.is_empty() {
            let env_array: Vec<Value> = environment_variables.into_iter()
                .map(|env_var| {
                    let mut env_map = serde_json::Map::new();
                    env_map.insert("name".to_string(), Value::String(env_var.name));
                    env_map.insert("description".to_string(), Value::String(env_var.description));
                    env_map.insert("required".to_string(), Value::Bool(env_var.required));
                    if let Some(default) = env_var.default {
                        env_map.insert("default".to_string(), Value::String(default));
                    }
                    Value::Object(env_map)
                })
                .collect();
            
            merged_deps.insert("environment_variables".to_string(), Value::Array(env_array));
        }
        
        Ok(merged_deps)
    }

    /*
    Applies conditional file inclusion based on service selections and conditions.
    Filters out files that don't meet their inclusion conditions.
    */
    async fn apply_conditional_inclusion(
        &self,
        files: Vec<ComposedFile>,
        services: &[ServiceSelection],
        composition_config: &Option<CompositionConfig>,
    ) -> EngineResult<Vec<ComposedFile>> {
        let mut filtered_files = Vec::new();

        // Create context for condition evaluation
        let mut context = HashMap::new();
        
        // Add service selections to context
        for service in services {
            let category_key = format!("{:?}", service.category).to_lowercase();
            context.insert(category_key, service.provider.clone());
            context.insert(format!("has_{}", format!("{:?}", service.category).to_lowercase()), "true".to_string());
        }

        for file in files {
            let should_include = self.evaluate_file_conditions(&file, &context, composition_config).await?;
            
            if should_include {
                filtered_files.push(file);
            }
        }

        Ok(filtered_files)
    }

    /*
    Evaluates whether a file should be included based on its conditions.
    */
    async fn evaluate_file_conditions(
        &self,
        file: &ComposedFile,
        context: &HashMap<String, String>,
        composition_config: &Option<CompositionConfig>,
    ) -> EngineResult<bool> {
        // Check global conditional files configuration
        if let Some(config) = composition_config {
            for conditional_file in &config.conditional_files {
                if file.path == PathBuf::from(&conditional_file.path) {
                    return self.evaluate_condition(&conditional_file.condition, context).await;
                }
            }
        }

        // Check file-specific conditions based on naming patterns
        self.evaluate_implicit_conditions(file, context).await
    }

    /*
    Evaluates a condition string against the current context.
    Supports basic conditions like:
    - "services.auth == 'clerk'"
    - "services.payments in ['stripe']" 
    - "has_auth && has_payments"
    */
    async fn evaluate_condition(
        &self,
        condition: &str,
        context: &HashMap<String, String>,
    ) -> EngineResult<bool> {
        let condition = condition.trim();

        // Handle AND conditions: "has_auth && has_payments"
        if condition.contains("&&") {
            let parts: Vec<&str> = condition.split("&&").collect();
            for part in parts {
                let result = self.evaluate_simple_condition(part.trim(), context)?;
                if !result {
                    return Ok(false);
                }
            }
            return Ok(true);
        }

        // Handle OR conditions: "has_auth || has_payments"
        if condition.contains("||") {
            let parts: Vec<&str> = condition.split("||").collect();
            for part in parts {
                let result = self.evaluate_simple_condition(part.trim(), context)?;
                if result {
                    return Ok(true);
                }
            }
            return Ok(false);
        }

        // Fallback to simple condition evaluation
        self.evaluate_simple_condition(condition, context)
    }

    /*
    Evaluates simple conditions without recursion.
    Handles basic equality, membership, and boolean conditions.
    */
    fn evaluate_simple_condition(
        &self,
        condition: &str,
        context: &HashMap<String, String>,
    ) -> EngineResult<bool> {
        let condition = condition.trim();

        // Handle equality conditions: "services.auth == 'clerk'"
        if condition.contains("==") {
            let parts: Vec<&str> = condition.split("==").collect();
            if parts.len() == 2 {
                let left = parts[0].trim();
                let right = parts[1].trim().trim_matches('\'').trim_matches('"');
                
                if left.starts_with("services.") {
                    let service_type = left.strip_prefix("services.").unwrap();
                    return Ok(context.get(service_type).map_or(false, |v| v == right));
                }
            }
        }

        // Handle 'in' conditions: "services.payments in ['stripe']"
        if condition.contains(" in ") {
            let parts: Vec<&str> = condition.split(" in ").collect();
            if parts.len() == 2 {
                let left = parts[0].trim();
                let right = parts[1].trim();
                
                if left.starts_with("services.") && right.starts_with('[') && right.ends_with(']') {
                    let service_type = left.strip_prefix("services.").unwrap();
                    let options: Vec<&str> = right[1..right.len()-1]
                        .split(',')
                        .map(|s| s.trim().trim_matches('\'').trim_matches('"'))
                        .collect();
                    
                    return Ok(context.get(service_type).map_or(false, |v| options.contains(&v.as_str())));
                }
            }
        }

        // Handle boolean conditions: "has_auth"
        if condition.starts_with("has_") {
            return Ok(context.get(condition).map_or(false, |v| v == "true"));
        }

        // Default: include file if condition is not recognized
        Ok(true)
    }

    /*
    Evaluates implicit conditions based on file paths and service selections.
    For example, files in auth/clerk/ are only included if clerk auth is selected.
    */
    async fn evaluate_implicit_conditions(
        &self,
        file: &ComposedFile,
        context: &HashMap<String, String>,
    ) -> EngineResult<bool> {
        match &file.source {
            FileSource::BaseTemplate => {
                // Base template files are always included
                Ok(true)
            }
            FileSource::Service { category, provider } => {
                // Service files are included if the service is selected
                let category_key = format!("{:?}", category).to_lowercase();
                Ok(context.get(&category_key).map_or(false, |selected| selected == provider))
            }
            FileSource::Merged => {
                // Merged files are always included
                Ok(true)
            }
        }
    }

    /*
    Collects environment variables required by all selected services.
    */
    async fn collect_environment_variables(&self, _services: &[ServiceSelection]) -> EngineResult<HashMap<String, String>> {
        // TODO: Implement environment variable collection from service configs
        Ok(HashMap::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    async fn create_test_structure() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create base template
        fs::create_dir_all(base_path.join("templates/test-app")).await.unwrap();
        fs::write(
            base_path.join("templates/test-app/anvil.yaml"),
            r#"
name: "test-app"
description: "Test application"
version: "1.0.0"
services:
  - name: "auth"
    category: "auth"
    prompt: "Choose auth provider"
    options: ["clerk", "auth0"]
    required: true
"#,
        ).await.unwrap();

        // Create shared services
        fs::create_dir_all(base_path.join("templates/shared/auth/clerk")).await.unwrap();
        fs::write(
            base_path.join("templates/shared/auth/clerk/middleware.ts"),
            "// Clerk middleware",
        ).await.unwrap();

        temp_dir
    }

    #[tokio::test]
    async fn test_compose_template() {
        let temp_dir = create_test_structure().await;
        let base_path = temp_dir.path();

        let engine = CompositionEngine::new(
            base_path.join("templates"),
            base_path.join("templates/shared"),
        );

        let services = vec![ServiceSelection {
            category: ServiceCategory::Auth,
            provider: "clerk".to_string(),
            config: HashMap::new(),
        }];

        let result = engine.compose_template("test-app", services).await;
        assert!(result.is_ok());

        let composed = result.unwrap();
        assert_eq!(composed.base_config.name, "test-app");
        assert!(!composed.files.is_empty());
    }
}