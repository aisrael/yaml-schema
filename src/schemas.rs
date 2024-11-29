use eyre::Result;
/// The schemas defined in the YAML schema language
use std::fmt;

mod any_of;
mod array;
mod bool_or_typed;
mod boolean;
mod r#const;
mod r#enum;
mod integer;
mod number;
mod object;
mod one_of;
mod string;

pub use array::ArraySchema;
pub use bool_or_typed::BoolOrTypedSchema;
pub use boolean::BooleanSchema;
pub use integer::IntegerSchema;
pub use number::NumberSchema;
pub use object::ObjectSchema;
pub use one_of::OneOfSchema;
pub use r#const::ConstSchema;
pub use r#enum::EnumSchema;
pub use string::StringSchema;

use crate::{Validator, YamlSchema};

#[derive(Debug, PartialEq)]
pub enum TypedSchema {
    Array(ArraySchema),
    Boolean,
    Null,
    Number(NumberSchema),
    Object(ObjectSchema),
    String(StringSchema),
}

impl From<YamlSchema> for TypedSchema {
    fn from(schema: YamlSchema) -> Self {
        match schema {
            YamlSchema::Array(a) => TypedSchema::Array(a),
            YamlSchema::BooleanSchema(_b) => TypedSchema::Boolean,
            YamlSchema::Number(n) => TypedSchema::Number(n),
            YamlSchema::Object(o) => TypedSchema::Object(o),
            YamlSchema::String(s) => TypedSchema::String(s),
            _ => unimplemented!("Can't convert YamlSchema to TypedSchema: {}!", schema),
        }
    }
}

/// A type value is either a string or an array of strings
#[derive(Debug, PartialEq)]
pub enum TypeValue {
    Single(serde_yaml::Value),
    Array(Vec<String>),
}

impl fmt::Display for TypedSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypedSchema::Array(a) => write!(f, "{}", a),
            TypedSchema::Boolean => write!(f, "type: boolean"),
            TypedSchema::Null => write!(f, "type: null"),
            TypedSchema::Number(n) => write!(f, "{}", n),
            TypedSchema::Object(o) => write!(f, "{}", o),
            TypedSchema::String(s) => write!(f, "{}", s),
        }
    }
}

impl Validator for TypedSchema {
    fn validate(&self, context: &crate::Context, value: &serde_yaml::Value) -> Result<()> {
        match self {
            TypedSchema::Array(a) => a.validate(context, value),
            TypedSchema::Boolean => Ok(()),
            TypedSchema::Null => {
                if !value.is_null() {
                    context.add_error(format!("Expected null, but got: {:?}", value));
                }
                Ok(())
            }
            TypedSchema::Number(n) => n.validate(context, value),
            TypedSchema::Object(o) => o.validate(context, value),
            TypedSchema::String(s) => s.validate(context, value),
        }
    }
}
