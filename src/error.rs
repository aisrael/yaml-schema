use thiserror::Error;

/// Unexpected errors that can occur during the validation of a YAML schema
#[derive(Debug, Error)]
pub enum Error {
    #[error("Not yet implemented!")]
    NotYetImplemented,
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("YAML parsing error: {0}")]
    YamlParsingError(#[from] saphyr::ScanError),
    #[error("Float parsing error: {0}")]
    FloatParsingError(#[from] std::num::ParseFloatError),
    #[error("Regex parsing error: {0}")]
    RegexParsingError(#[from] regex::Error),
    #[error("Unsupported type '{0}'!")]
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
            return Err($crate::Error::FailFast);
        }
    };
}

#[macro_export]
macro_rules! unsupported_type {
    ($s:literal, $($e:expr),+) => {
        $crate::Error::UnsupportedType(format!($s, $($e),+))
    };
    ($e:expr) => {
        $crate::Error::UnsupportedType($e)
    };
}

#[macro_export]
macro_rules! generic_error {
    ($s:literal, $($e:expr),+) => {
        Err($crate::Error::GenericError(format!($s, $($e),+)))
    };
    ($s:literal) => {
        $crate::Error::GenericError($s.to_string())
    };
}
