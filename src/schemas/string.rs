use std::fmt;

use crate::Result;
use crate::{validation::strings::validate_string, Context, Validator};

/// A string schema
#[derive(Debug, PartialEq, Default)]
pub struct StringSchema {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
}

impl fmt::Display for StringSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "String {:?}", self)
    }
}

impl Validator for StringSchema {
    fn validate(&self, context: &Context, value: &serde_yaml::Value) -> Result<()> {
        match validate_string(
            self.min_length,
            self.max_length,
            self.pattern.as_ref(),
            value,
        ) {
            Ok(errors) => {
                if !errors.is_empty() {
                    for error in errors {
                        context.add_error(error);
                    }
                }
                Ok(())
            }
            Err(e) => generic_error!("{}", e),
        }
    }
}
