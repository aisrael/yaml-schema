use std::fmt;

use crate::format_serde_yaml_value;
use crate::Context;
use crate::Result;
use crate::Validator;

/// An enum schema represents a set of constant values
#[derive(Debug, PartialEq)]
pub struct EnumSchema {
    pub r#enum: Vec<serde_yaml::Value>,
}

impl fmt::Display for EnumSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Enum {:?}", self.r#enum)
    }
}

impl Validator for EnumSchema {
    fn validate(&self, context: &Context, value: &serde_yaml::Value) -> Result<()> {
        if !self.r#enum.contains(value) {
            let value_str = format_serde_yaml_value(value);
            let enum_values = self
                .r#enum
                .iter()
                .map(format_serde_yaml_value)
                .collect::<Vec<String>>()
                .join(", ");
            let error = format!("Value {} is not in the enum: [{}]", value_str, enum_values);
            context.add_error(error);
        }
        Ok(())
    }
}
