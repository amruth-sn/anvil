use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tera::Tera;
use walkdir::WalkDir;
use serde_yaml::Value;
use chrono::{DateTime, Utc};

use crate::config::TemplateConfig;
use crate::error::{EngineError, EngineResult};

#[derive(Debug, Clone)]
pub struct Context {
    variables: HashMap<String, Value>,
    features: Vec<String>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            features: Vec::new(),
        }
    }

    pub fn builder() -> ContextBuilder {
        ContextBuilder::new()
    }

    pub fn add_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn add_feature(&mut self, feature: String) {
        if !self.features.contains(&feature) {
            self.features.push(feature);
        }
    }

    pub fn has_feature(&self, feature: &str) -> bool {
        self.features.contains(&feature.to_string())
    }

    pub fn variables(&self) -> &HashMap<String, Value> {
        &self.variables
    }

    pub fn features(&self) -> &[String] {
        &self.features
    }

    pub fn to_tera_context(&self) -> tera::Context {
        let mut tera_context = tera::Context::new();
        
        for (key, value) in &self.variables {
            tera_context.insert(key, value);
        }
        
        tera_context.insert("features", &self.features);
        for feature in &self.features {
            tera_context.insert(&format!("feature_{}", feature), &true);
        }
        
        tera_context
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct ContextBuilder {
    context: Context,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            context: Context::new(),
        }
    }

    pub fn variable(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.context.add_variable(name.into(), value.into());
        self
    }

    pub fn feature(mut self, feature: impl Into<String>) -> Self {
        self.context.add_feature(feature.into());
        self
    }

    pub fn build(self) -> Context {
        self.context
    }
}

#[derive(Debug, Clone)]
pub struct TemplateFile {
    pub source_path: PathBuf,
    pub relative_path: PathBuf,
    pub output_path: PathBuf,
    pub content: String,
}

#[derive(Debug)]
pub struct ProcessedTemplate {
    pub files: Vec<ProcessedFile>,
}

#[derive(Debug)]
pub struct ProcessedFile {
    pub output_path: PathBuf,
    pub content: String,
    pub executable: bool,
}

pub struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    pub fn new() -> EngineResult<Self> {
        let mut tera = Tera::new("templates/**/*").map_err(EngineError::ProcessingError)?;
        
        tera.register_filter("snake_case", Self::snake_case_filter);
        tera.register_filter("pascal_case", Self::pascal_case_filter);
        tera.register_filter("kebab_case", Self::kebab_case_filter);
        tera.register_filter("rust_module_name", Self::rust_module_name_filter);
        
        Ok(Self { tera })
    }

    pub fn new_for_testing() -> EngineResult<Self> {
        let mut tera = Tera::default();
        
        tera.register_filter("snake_case", Self::snake_case_filter);
        tera.register_filter("pascal_case", Self::pascal_case_filter);
        tera.register_filter("kebab_case", Self::kebab_case_filter);
        tera.register_filter("rust_module_name", Self::rust_module_name_filter);
        
        Ok(Self { tera })
    }

    pub fn discover_template_files(
        &self,
        template_dir: &Path,
    ) -> EngineResult<Vec<TemplateFile>> {
        let mut files = Vec::new();
        
        for entry in WalkDir::new(template_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let source_path = entry.path().to_path_buf();
            
            if source_path.file_name().and_then(|n| n.to_str()) == Some("anvil.yaml") {
                continue;
            }
            
            let relative_path = source_path
                .strip_prefix(template_dir)
                .map_err(|_| EngineError::invalid_config("Invalid template path"))?
                .to_path_buf();
            
            let output_path = if relative_path.extension().and_then(|e| e.to_str()) == Some("tera") {
                // Remove .tera extension: package.json.tera -> package.json
                let file_name = relative_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .trim_end_matches(".tera");
                relative_path.with_file_name(file_name)
            } else {
                relative_path.clone()
            };
            
            let content = std::fs::read_to_string(&source_path)
                .map_err(|e| EngineError::file_error(&source_path, e))?;
            
            files.push(TemplateFile {
                source_path,
                relative_path,
                output_path,
                content,
            });
        }
        
        Ok(files)
    }

    pub async fn process_template(
        &mut self,
        template_dir: &Path,
        context: &Context,
    ) -> EngineResult<ProcessedTemplate> {
        let template_files = self.discover_template_files(template_dir)?;
        let tera_context = context.to_tera_context();
        
        let mut processed_files = Vec::new();
        
        for template_file in template_files {
            let processed_content = if template_file.source_path.extension().and_then(|e| e.to_str()) == Some("tera") {
                self.tera.render_str(&template_file.content, &tera_context)
                    .map_err(EngineError::ProcessingError)?
            } else {
                template_file.content
            };
            
            let executable = self.should_be_executable(&template_file.output_path);
            
            processed_files.push(ProcessedFile {
                output_path: template_file.output_path,
                content: processed_content,
                executable,
            });
        }
        
        Ok(ProcessedTemplate {
            files: processed_files,
        })
    }

    /*
    Processes a ComposedTemplate (from composition engine) into a ProcessedTemplate
    by rendering all template content with the given context.
    */
    pub async fn process_composed_template(
        &mut self,
        composed: crate::composition::ComposedTemplate,
        context: &Context,
    ) -> EngineResult<ProcessedTemplate> {
        // Build comprehensive shared context
        let tera_context = self.build_shared_context(context, &composed)?;
        
        let mut processed_files = Vec::new();
        
        for composed_file in composed.files {
            let processed_content = if composed_file.is_template {
                self.tera.render_str(&composed_file.content, &tera_context)
                    .map_err(EngineError::ProcessingError)?
            } else {
                composed_file.content
            };
            
            let executable = self.should_be_executable(&composed_file.path);
            
            processed_files.push(ProcessedFile {
                output_path: composed_file.path,
                content: processed_content,
                executable,
            });
        }
        
        Ok(ProcessedTemplate {
            files: processed_files,
        })
    }

    pub fn render_string(&mut self, template: &str, context: &Context) -> EngineResult<String> {
        let tera_context = context.to_tera_context();
        self.tera.render_str(template, &tera_context)
            .map_err(EngineError::ProcessingError)
    }

    pub fn validate_context(
        &self,
        context: &Context,
        config: &TemplateConfig,
    ) -> EngineResult<()> {
        for variable in &config.variables {
            if variable.required {
                if !context.variables.contains_key(&variable.name) {
                    return Err(EngineError::variable_error(
                        &variable.name,
                        "Required variable not provided",
                    ));
                }
            }
            
            if let Some(value) = context.get_variable(&variable.name) {
                variable.validate_value(value)?;
            }
        }
        
        Ok(())
    }

    fn should_be_executable(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            matches!(extension, "sh" | "py" | "rb" | "pl")
        } else {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                matches!(filename, "gradlew" | "mvnw" | "install" | "configure" | "bootstrap")
            } else {
                false
            }
        }
    }


    fn snake_case_filter(value: &tera::Value, _: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        let s = value.as_str().ok_or_else(|| tera::Error::msg("Value must be a string"))?;
        let snake_case = s
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if c.is_uppercase() && i > 0 {
                    format!("_{}", c.to_lowercase())
                } else {
                    c.to_lowercase().to_string()
                }
            })
            .collect::<String>()
            .replace(' ', "_")
            .replace('-', "_");
        Ok(tera::Value::String(snake_case))
    }

    fn pascal_case_filter(value: &tera::Value, _: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        let s = value.as_str().ok_or_else(|| tera::Error::msg("Value must be a string"))?;
        
        // First, split by common delimiters and handle camelCase/PascalCase
        let mut words = Vec::new();
        for segment in s.split(&[' ', '_', '-'][..]) {
            if segment.is_empty() {
                continue;
            }
            
            // Split camelCase/PascalCase by detecting uppercase letters
            let mut current_word = String::new();
            for ch in segment.chars() {
                if ch.is_uppercase() && !current_word.is_empty() {
                    words.push(current_word.clone());
                    current_word.clear();
                }
                current_word.push(ch);
            }
            if !current_word.is_empty() {
                words.push(current_word);
            }
        }
        
        let pascal_case = words
            .iter()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            })
            .collect::<String>();
        Ok(tera::Value::String(pascal_case))
    }

    fn kebab_case_filter(value: &tera::Value, _: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        let s = value.as_str().ok_or_else(|| tera::Error::msg("Value must be a string"))?;
        let kebab_case = s
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if c.is_uppercase() && i > 0 {
                    format!("-{}", c.to_lowercase())
                } else {
                    c.to_lowercase().to_string()
                }
            })
            .collect::<String>()
            .replace(' ', "-")
            .replace('_', "-");
        Ok(tera::Value::String(kebab_case))
    }

    /*
    Builds a comprehensive shared context that combines user variables, service context,
    template metadata, and build information for template rendering.
    */
    fn build_shared_context(
        &self,
        user_context: &Context,
        composed: &crate::composition::ComposedTemplate,
    ) -> EngineResult<tera::Context> {
        let mut tera_context = user_context.to_tera_context();
        
        // Add template metadata
        tera_context.insert("template", &serde_json::json!({
            "name": composed.base_config.name,
            "description": composed.base_config.description,
            "version": composed.base_config.version,
            "min_anvil_version": composed.base_config.min_anvil_version
        }));
        
        // Add build context
        let now: DateTime<Utc> = Utc::now();
        tera_context.insert("build", &serde_json::json!({
            "timestamp": now.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            "timestamp_iso": now.to_rfc3339(),
            "year": now.format("%Y").to_string(),
            "generator": "Anvil Template Engine",
            "generator_version": env!("CARGO_PKG_VERSION")
        }));
        
        // Add merged dependencies to context for template rendering
        tera_context.insert("merged_dependencies", &composed.merged_dependencies);
        
        // Add environment variables to context
        tera_context.insert("environment_variables", &composed.environment_variables);
        
        // Add service context for dependency injection (flattened for Tera compatibility)
        for (service_name, service_info) in &composed.service_context.services {
            tera_context.insert(&format!("service_{}", service_name.to_lowercase()), &service_info.provider);
            for (export_key, export_value) in &service_info.exports {
                tera_context.insert(&format!("{}_{}", service_name.to_lowercase(), export_key), export_value);
            }
        }
        
        // Add shared config
        for (key, value) in &composed.service_context.shared_config {
            tera_context.insert(key, value);
        }
        
        // Add service summary for easy template access
        let service_summary: Vec<serde_json::Value> = composed.service_context.services.iter()
            .map(|(name, info)| serde_json::json!({
                "category": name,
                "provider": info.provider,
                "has_config": !info.config.is_empty()
            }))
            .collect();
        tera_context.insert("active_services", &service_summary);
        
        // Add utility flags
        tera_context.insert("has_services", &(!composed.service_context.services.is_empty()));
        tera_context.insert("has_dependencies", &(!composed.merged_dependencies.is_empty()));
        tera_context.insert("has_environment_variables", &(!composed.environment_variables.is_empty()));
        
        Ok(tera_context)
    }

    fn rust_module_name_filter(value: &tera::Value, _: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        let s = value.as_str().ok_or_else(|| tera::Error::msg("Value must be a string"))?;
        let module_name = s
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if c.is_uppercase() && i > 0 {
                    format!("_{}", c.to_lowercase())
                } else if c.is_alphanumeric() || c == '_' {
                    c.to_lowercase().to_string()
                } else {
                    "_".to_string()
                }
            })
            .collect::<String>()
            .replace(' ', "_")
            .replace('-', "_");
        
        let module_name = if module_name.chars().next().map_or(false, |c| c.is_numeric()) {
            format!("_{}", module_name)
        } else {
            module_name
        };
        
        Ok(tera::Value::String(module_name))
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default template engine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_context_builder() {
        let context = Context::builder()
            .variable("project_name", "test-project")
            .variable("author", "Test Author")
            .feature("database")
            .feature("auth")
            .build();
        
        assert_eq!(context.get_variable("project_name").unwrap().as_str().unwrap(), "test-project");
        assert_eq!(context.get_variable("author").unwrap().as_str().unwrap(), "Test Author");
        assert!(context.has_feature("database"));
        assert!(context.has_feature("auth"));
        assert!(!context.has_feature("nonexistent"));
    }

    #[test]
    fn test_template_engine_creation() {
        let _engine = TemplateEngine::new_for_testing().unwrap();
    }

    #[test]
    fn test_render_string() {
        let mut engine = TemplateEngine::new_for_testing().unwrap();
        let context = Context::builder()
            .variable("name", "World")
            .build();
        
        let result = engine.render_string("Hello, {{ name }}!", &context).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_custom_filters() {
        let mut engine = TemplateEngine::new_for_testing().unwrap();
        let context = Context::builder()
            .variable("project_name", "MyAwesomeProject")
            .build();
        
        let result = engine.render_string("{{ project_name | snake_case }}", &context).unwrap();
        assert_eq!(result, "my_awesome_project");
        
        let result = engine.render_string("{{ project_name | pascal_case }}", &context).unwrap();
        assert_eq!(result, "MyAwesomeProject");
        
        let result = engine.render_string("{{ project_name | kebab_case }}", &context).unwrap();
        assert_eq!(result, "my-awesome-project");
        
        let result = engine.render_string("{{ project_name | rust_module_name }}", &context).unwrap();
        assert_eq!(result, "my_awesome_project");
    }

    #[tokio::test]
    async fn test_template_file_discovery() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("template");
        fs::create_dir_all(&template_dir).unwrap();
        
        fs::write(template_dir.join("file.txt.tera"), "Hello {{ name }}").unwrap();
        fs::write(template_dir.join("static.md"), "# README").unwrap();
        
        let engine = TemplateEngine::new_for_testing().unwrap();
        let files = engine.discover_template_files(&template_dir).unwrap();
        
        
        let template_file = files.iter().find(|f| f.relative_path.to_str().unwrap() == "file.txt.tera").unwrap();
        assert_eq!(template_file.output_path, PathBuf::from("file.txt"));
        
        let static_file = files.iter().find(|f| f.relative_path.to_str().unwrap() == "static.md").unwrap();
        assert_eq!(static_file.output_path, PathBuf::from("static.md"));
    }

    #[tokio::test]
    async fn test_template_processing() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("template");
        std::fs::create_dir_all(&template_dir).unwrap();
        
        std::fs::write(
            template_dir.join("main.rs.tera"),
            r#"fn main() {
    println!("Hello from {{ project_name | pascal_case }}!");
}
"#,
        ).unwrap();
        
        let mut engine = TemplateEngine::new_for_testing().unwrap();
        let context = Context::builder()
            .variable("project_name", "my-project")
            .build();
        
        let result = engine.process_template(&template_dir, &context).await.unwrap();
        
        assert_eq!(result.files.len(), 1);
        let file = &result.files[0];
        assert_eq!(file.output_path, PathBuf::from("main.rs"));
        assert!(file.content.contains("Hello from MyProject!"));
    }
}