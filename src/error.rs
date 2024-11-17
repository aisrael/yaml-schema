use thiserror::Error;

/// Unexpected errors that can occur during the validation of a YAML schema
#[derive(Clone, Debug, Error, PartialEq)]
pub enum YamlSchemaError {
    #[error("Not yet implemented!")]
    NotYetImplemented,
    #[error("YAML parsing error: {0}")]
    YamlParsingError(#[from] yaml_rust2::ScanError),
    #[error("Generic YAML schema error: {0}")]
    GenericError(String),
    #[error("Fail fast signal")]
    FailFast,
}

#[macro_export]
macro_rules! fail_fast {
    ($context:expr) => {
        if $context.fail_fast {
            return Err(YamlSchemaError::FailFast);
        }
    };
}

#[macro_export]
macro_rules! generic_error {
    ($s:literal, $($e:expr),+) => {
        YamlSchemaError::GenericError(format!($s, $($e),+))
    };
    ($s:literal) => {
        YamlSchemaError::GenericError($s.to_string())
    };
}
