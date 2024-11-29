use thiserror::Error;

/// Unexpected errors that can occur during the validation of a YAML schema
#[derive(Clone, Debug, Error, PartialEq)]
pub enum Error {
    #[error("Not yet implemented!")]
    NotYetImplemented,
    #[error("YAML parsing error: {0}")]
    YamlParsingError(#[from] yaml_rust2::ScanError),
    #[error("Regex parsing error: {0}")]
    RegexParsingError(#[from] regex::Error),
    #[error("Unsupported type: {0}")]
    UnsupportedType(String),
    #[error("Generic YAML schema error: {0}")]
    GenericError(String),
    #[error("Fail fast signal")]
    FailFast,
}

#[macro_export]
macro_rules! fail_fast {
    ($context:expr) => {
        if $context.fail_fast {
            return Err(Error::FailFast);
        }
    };
}

#[macro_export]
macro_rules! unsupported_type {
    ($s:literal, $($e:expr),+) => {
        Err(Error::UnsupportedType(format!($s, $($e),+)))
    };
    ($e:expr) => {
        Err(Error::UnsupportedType($e))
    };
}

#[macro_export]
macro_rules! generic_error {
    ($s:literal, $($e:expr),+) => {
        Err(Error::GenericError(format!($s, $($e),+)))
    };
    ($s:literal) => {
        Error::GenericError($s.to_string())
    };
}
