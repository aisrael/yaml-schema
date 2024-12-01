/// Validation engine for YamlSchema
mod context;
pub mod objects;
pub mod one_of;
pub mod strings;

pub use context::Context;
use log::debug;

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
impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.error)
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
            YamlSchema::BooleanLiteral(boolean) => {
                if !*boolean {
                    context.add_error("Schema is `false`!".to_string());
                }
                Ok(())
            }
            YamlSchema::BooleanSchema => validate_boolean_schema(context, value),
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

fn validate_boolean_schema(context: &Context, value: &serde_yaml::Value) -> Result<()> {
    if !value.is_bool() {
        context.add_error(format!("Expected: boolean, found: {:?}", value));
    }
    Ok(())
}
