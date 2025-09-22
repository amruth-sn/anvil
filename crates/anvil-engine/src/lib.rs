pub mod config;
pub mod engine;
pub mod error;
pub mod generator;

pub use config::{TemplateConfig, TemplateVariable, VariableType};
pub use engine::{TemplateEngine, Context};
pub use error::{EngineError, EngineResult};
pub use generator::FileGenerator;