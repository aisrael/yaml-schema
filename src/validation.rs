/// Validation engine for YamlSchema
use std::fmt::Display;

mod context;
pub mod objects;
pub mod one_of;
pub mod strings;

pub use context::Context;
use log::{debug, error};
use one_of::validate_one_of;

use crate::format_serde_yaml_value;
use crate::ConstSchema;
use crate::EnumSchema;
use crate::OneOfSchema;
use crate::Result;
use crate::YamlSchema;

/// A trait for validating a value against a schema
pub trait Validator {
    fn validate(&self, context: &Context, value: &serde_yaml::Value) -> Result<()>;
}

/// A validation error simply contains a path and an error message
#[derive(Debug)]
pub struct ValidationError {
    /// The path to the value that caused the error
    pub path: String,
    /// The error message
    pub error: String,
}

/// Display this ValidationErrors as "{path}: {error}"
impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.error)
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

impl Validator for OneOfSchema {
    fn validate(&self, context: &Context, value: &serde_yaml::Value) -> Result<()> {
        let one_of_is_valid = validate_one_of(context, &self.one_of, value)?;
        if !one_of_is_valid {
            error!("OneOf: None of the schemas in `oneOf` matched!");
            context.add_error("None of the schemas in `oneOf` matched!");
            fail_fast!(context);
        }
        Ok(())
    }
}

impl Validator for YamlSchema {
    fn validate(&self, context: &Context, value: &serde_yaml::Value) -> Result<()> {
        debug!("YamlSchema: self: {}", self);
        debug!("YamlSchema: Validating value: {:?}", value);
        match self {
            YamlSchema::Empty => Ok(()),
            YamlSchema::TypeNull => {
                if !value.is_null() {
                    context.add_error(format!("Expected null, but got: {:?}", value));
                }
                Ok(())
            }
            YamlSchema::Boolean(boolean) => {
                if !*boolean {
                    context.add_error("Schema is `false`!".to_string());
                }
                Ok(())
            }
            YamlSchema::BooleanSchema(boolean_schema) => boolean_schema.validate(context, value),
            YamlSchema::Const(const_schema) => const_schema.validate(context, value),
            YamlSchema::Enum(enum_schema) => enum_schema.validate(context, value),
            YamlSchema::Integer(integer_schema) => integer_schema.validate(context, value),
            YamlSchema::Object(object_schema) => object_schema.validate(context, value),
            YamlSchema::OneOf(one_of_schema) => one_of_schema.validate(context, value),
            YamlSchema::String(string_schema) => string_schema.validate(context, value),
            YamlSchema::Number(number_schema) => number_schema.validate(context, value),
            YamlSchema::Array(array_schema) => array_schema.validate(context, value),
        }
    }
}
