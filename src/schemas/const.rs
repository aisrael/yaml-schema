use log::debug;

use super::Validator;
use crate::Context;
use crate::Result;

/// A const schema represents a constant value
#[derive(Debug, PartialEq)]
pub struct ConstSchema {
    pub r#const: serde_yaml::Value,
}

impl std::fmt::Display for ConstSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Const {:?}", self.r#const)
    }
}

impl Validator for ConstSchema {
    fn validate(&self, context: &Context, value: &serde_yaml::Value) -> Result<()> {
        debug!(
            "Validating value: {:?} against const: {:?}",
            value, self.r#const
        );
        let expected_value = &self.r#const;
        if expected_value != value {
            let error = format!(
                "Const validation failed, expected: {:?}, got: {:?}",
                expected_value, value
            );
            context.add_error(error);
        }
        Ok(())
    }
}
