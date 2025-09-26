use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Template not found: {name}")]
    TemplateNotFound { name: String },

    #[error("Invalid template configuration: {reason}")]
    InvalidConfig { reason: String },

    #[error("File operation failed: {path}")]
    FileError { 
        path: PathBuf,
        #[source]
        source: std::io::Error 
    },

    #[error("Template processing failed")]
    ProcessingError(#[from] tera::Error),

    #[error("YAML parsing failed")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Variable validation failed: {variable}: {reason}")]
    VariableError { variable: String, reason: String },

    #[error("Feature dependency not met: {feature} requires {dependency}")]
    FeatureDependencyError { feature: String, dependency: String },

    #[error("Template composition failed: {reason}")]
    CompositionError { reason: String },
}

pub type EngineResult<T> = Result<T, EngineError>;

impl EngineError {
    pub fn template_not_found(name: impl Into<String>) -> Self {
        Self::TemplateNotFound { name: name.into() }
    }
    
    pub fn invalid_config(reason: impl Into<String>) -> Self {
        Self::InvalidConfig { reason: reason.into() }
    }

    pub fn file_error(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::FileError { 
            path: path.into(), 
            source 
        }
    }

    pub fn variable_error(variable: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::VariableError {
            variable: variable.into(),
            reason: reason.into(),
        }
    }

    pub fn feature_dependency_error(feature: impl Into<String>, dependency: impl Into<String>) -> Self {
        Self::FeatureDependencyError {
            feature: feature.into(),
            dependency: dependency.into(),
        }
    }

    pub fn composition_error(reason: impl Into<String>) -> Self {
        Self::CompositionError { reason: reason.into() }
    }
}