use thiserror::Error;

#[derive(Error, Debug)]
pub enum YamlSchemaError {
    #[error("Not yet implemented!")]
    NotYetImplemented,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("YAML parsing error: {0}")]
    YamlParsingError(#[from] yaml_rust2::ScanError),
    #[error("Generic YAML schema error: {0}")]
    GenericError(String),
}

#[macro_export]
macro_rules! not_yet_implemented {
    () => {
        return Err(YamlSchemaError::NotYetImplemented);
    };
}
