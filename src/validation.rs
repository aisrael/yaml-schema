/// Validation engine for YamlSchema
use std::fmt::Display;

mod context;
pub mod objects;
pub mod one_of;
pub mod strings;

pub use context::Context;

/// A validation error simply contains a path and an error message
#[derive(Debug)]
pub struct ValidationError {
    /// The path to the value that caused the error
    pub path: String,
    /// The error message
    pub error: String,
}

/// Display this ValidationErrors as "{path}: {error}"
impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.error)
    }
}
