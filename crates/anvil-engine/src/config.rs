use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::{EngineError, EngineResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TemplateConfig {
    pub name: String,
    pub description: String,
    pub version: String,
    
    #[serde(default)]
    pub variables: Vec<TemplateVariable>,
    
    #[serde(default)]
    pub features: Vec<Feature>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<Hooks>,
    
    #[serde(default = "default_min_anvil_version")]
    pub min_anvil_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    #[serde(rename = "type")]
    pub var_type: VariableType,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_yaml::Value>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VariableType {
    String { 
        #[serde(default)]
        min_length: usize,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_length: Option<usize>,
    },
    Boolean,
    Choice { options: Vec<String> },
    Number { 
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<i64>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_when: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hooks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_generate: Option<Vec<HookCommand>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_generate: Option<Vec<HookCommand>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookCommand {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl TemplateConfig {
    pub async fn from_file(path: &std::path::Path) -> EngineResult<Self> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| EngineError::file_error(path, e))?;
        
        let config: TemplateConfig = serde_yaml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> EngineResult<()> {
        if self.name.is_empty() {
            return Err(EngineError::invalid_config("Template name cannot be empty"));
        }
        
        if self.description.is_empty() {
            return Err(EngineError::invalid_config("Template description cannot be empty"));
        }
        
        semver::Version::parse(&self.version)
            .map_err(|_| EngineError::invalid_config("Invalid version format"))?;
        
        semver::Version::parse(&self.min_anvil_version)
            .map_err(|_| EngineError::invalid_config("Invalid min_anvil_version format"))?;
        
        for variable in &self.variables {
            variable.validate()?;
        }
        
        for feature in &self.features {
            feature.validate()?;
        }
        
        Ok(())
    }

    pub fn get_variable(&self, name: &str) -> Option<&TemplateVariable> {
        self.variables.iter().find(|v| v.name == name)
    }

    pub fn get_feature(&self, name: &str) -> Option<&Feature> {
        self.features.iter().find(|f| f.name == name)
    }
}

impl TemplateVariable {
    pub fn validate(&self) -> EngineResult<()> {
        if self.name.is_empty() {
            return Err(EngineError::invalid_config("Variable name cannot be empty"));
        }
        
        if self.prompt.is_empty() {
            return Err(EngineError::invalid_config(format!("Variable '{}' must have a prompt", self.name)));
        }
        
        match &self.var_type {
            VariableType::String { min_length, max_length } => {
                if let Some(max) = max_length {
                    if *min_length > *max {
                        return Err(EngineError::invalid_config(
                            format!("Variable '{}': min_length cannot be greater than max_length", self.name)
                        ));
                    }
                }
            }
            VariableType::Choice { options } => {
                if options.is_empty() {
                    return Err(EngineError::invalid_config(
                        format!("Variable '{}': choice type must have at least one option", self.name)
                    ));
                }
            }
            VariableType::Number { min, max } => {
                if let (Some(min_val), Some(max_val)) = (min, max) {
                    if min_val > max_val {
                        return Err(EngineError::invalid_config(
                            format!("Variable '{}': min cannot be greater than max", self.name)
                        ));
                    }
                }
            }
            VariableType::Boolean => {}
        }
        
        Ok(())
    }

    pub fn validate_value(&self, value: &serde_yaml::Value) -> EngineResult<()> {
        match (&self.var_type, value) {
            (VariableType::String { min_length, max_length }, serde_yaml::Value::String(s)) => {
                if s.len() < *min_length {
                    return Err(EngineError::variable_error(&self.name, 
                        format!("String too short (minimum {} characters)", min_length)));
                }
                if let Some(max) = max_length {
                    if s.len() > *max {
                        return Err(EngineError::variable_error(&self.name,
                            format!("String too long (maximum {} characters)", max)));
                    }
                }
            }
            (VariableType::Boolean, serde_yaml::Value::Bool(_)) => {}
            (VariableType::Number { min, max }, serde_yaml::Value::Number(n)) => {
                if let Some(i) = n.as_i64() {
                    if let Some(min_val) = min {
                        if i < *min_val {
                            return Err(EngineError::variable_error(&self.name,
                                format!("Number too small (minimum {})", min_val)));
                        }
                    }
                    if let Some(max_val) = max {
                        if i > *max_val {
                            return Err(EngineError::variable_error(&self.name,
                                format!("Number too large (maximum {})", max_val)));
                        }
                    }
                }
            }
            (VariableType::Choice { options }, serde_yaml::Value::String(s)) => {
                if !options.contains(s) {
                    return Err(EngineError::variable_error(&self.name,
                        format!("Invalid choice '{}'. Valid options: {}", s, options.join(", "))));
                }
            }
            _ => {
                return Err(EngineError::variable_error(&self.name,
                    format!("Value type mismatch for variable type {:?}", self.var_type)));
            }
        }
        Ok(())
    }
}

impl Feature {
    pub fn validate(&self) -> EngineResult<()> {
        if self.name.is_empty() {
            return Err(EngineError::invalid_config("Feature name cannot be empty"));
        }
        
        if self.description.is_empty() {
            return Err(EngineError::invalid_config(format!("Feature '{}' must have a description", self.name)));
        }
        
        Ok(())
    }
}

fn default_min_anvil_version() -> String {
    "0.1.0".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_valid_config_parsing() {
        let yaml_content = r#"
name: "test-template"
description: "A test template"
version: "1.0.0"
variables:
  - name: "project_name"
    type: "string"
    prompt: "Project name?"
    required: true
    min_length: 1
features:
  - name: "database"
    description: "Database integration"
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(yaml_content.as_bytes()).unwrap();
        
        let config = TemplateConfig::from_file(temp_file.path()).await.unwrap();
        assert_eq!(config.name, "test-template");
        assert_eq!(config.variables.len(), 1);
        assert_eq!(config.features.len(), 1);
    }

    #[test]
    fn test_config_validation() {
        let mut config = TemplateConfig {
            name: "test".to_string(),
            description: "Test template".to_string(),
            version: "1.0.0".to_string(),
            variables: vec![],
            features: vec![],
            hooks: None,
            min_anvil_version: "0.1.0".to_string(),
        };
        
        assert!(config.validate().is_ok());
        
        config.name = "".to_string();
        assert!(config.validate().is_err());
        
        config.name = "test".to_string();
        config.version = "invalid-version".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_variable_validation() {
        let variable = TemplateVariable {
            name: "test_var".to_string(),
            var_type: VariableType::String { min_length: 1, max_length: Some(10) },
            prompt: "Test variable?".to_string(),
            default: None,
            required: true,
        };
        
        assert!(variable.validate().is_ok());
        
        assert!(variable.validate_value(&serde_yaml::Value::String("test".to_string())).is_ok());
        assert!(variable.validate_value(&serde_yaml::Value::String("".to_string())).is_err());
        assert!(variable.validate_value(&serde_yaml::Value::String("this_is_too_long".to_string())).is_err());
    }
}