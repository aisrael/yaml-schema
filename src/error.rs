use thiserror::Error;

#[derive(Error, Debug)]
pub enum YamlSchemaError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("YAML parsing error: {0}")]
    YamlParsingError(#[from] yaml_rust2::ScanError),
    #[error("Generic YAML schema error: {0}")]
    GenericError(String),
}
