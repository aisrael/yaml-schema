/// Validation engine for YamlSchema
pub mod strings;

/// A validation error simply contains a path and an error message
#[derive(Debug)]
pub struct ValidationError {
    /// The path to the value that caused the error
    pub path: String,
    /// The error message
    pub error: String,
}
