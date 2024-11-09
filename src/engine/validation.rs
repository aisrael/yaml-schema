/// Validation engine for YamlSchema

/// A validation error simply contains a path and an error message
#[derive(Debug)]
pub struct ValidationError {
    pub path: String,
    pub error: String,
}
