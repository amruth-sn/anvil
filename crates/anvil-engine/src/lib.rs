pub mod config;
pub mod composition;
pub mod engine;
pub mod error;
pub mod generator;

pub use config::{
    TemplateConfig, TemplateVariable, VariableType, ServiceDefinition, 
    ServiceCategory, CompositionConfig, FileMergingStrategy, DependencyResolution, ConditionalFile,
    ServiceConfig, ServiceDependencies, EnvironmentVariable, ServiceFile,
    ServiceCombination, ServicePromptType, ServicePrompt
};
pub use composition::{CompositionEngine, ServiceSelection, ComposedTemplate, ComposedFile, FileSource};
pub use engine::{TemplateEngine, Context};
pub use error::{EngineError, EngineResult};
pub use generator::FileGenerator;