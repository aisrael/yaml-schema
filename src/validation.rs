/// Validation engine for YamlSchema
use std::fmt::Display;

mod context;
pub mod objects;
pub mod one_of;
pub mod strings;

pub use context::Context;
use log::{debug, error};
use one_of::validate_one_of;

use crate::{format_serde_yaml_value, ConstSchema, EnumSchema, OneOfSchema, YamlSchemaError};

/// A trait for validating a value against a schema
pub trait Validator {
    fn validate(&self, context: &Context, value: &serde_yaml::Value)
        -> Result<(), YamlSchemaError>;
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
    fn validate(
        &self,
        context: &Context,
        value: &serde_yaml::Value,
    ) -> Result<(), YamlSchemaError> {
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
    fn validate(
        &self,
        context: &Context,
        value: &serde_yaml::Value,
    ) -> Result<(), YamlSchemaError> {
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
    fn validate(
        &self,
        context: &Context,
        value: &serde_yaml::Value,
    ) -> Result<(), YamlSchemaError> {
        let one_of_is_valid = validate_one_of(context, &self.one_of, value)?;
        if !one_of_is_valid {
            error!("OneOf: None of the schemas in `oneOf` matched!");
            context.add_error("None of the schemas in `oneOf` matched!");
            fail_fast!(context);
        }
        Ok(())
    }
}
