use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum YamlSchemaError {
    #[error("Not yet implemented!")]
    NotYetImplemented,
    #[error("YAML parsing error: {0}")]
    YamlParsingError(#[from] yaml_rust2::ScanError),
    #[error("Generic YAML schema error: {0}")]
    GenericError(String),
}

#[macro_export]
macro_rules! generic_error {
    ($s:literal, $($e:expr),+) => {
        Err(YamlSchemaError::GenericError(format!($s, $($e),+)))
    };
    ($s:literal) => {
        Err(YamlSchemaError::GenericError($s.to_string()))
    };
}

#[macro_export]
macro_rules! not_yet_implemented {
    () => {
        Err(YamlSchemaError::NotYetImplemented)
    };
}
