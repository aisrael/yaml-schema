/// The schemas defined in the YAML schema language
use serde::{Deserialize, Serialize};
use std::fmt;

mod any_of;
pub mod array;
pub mod number;
pub mod object;
pub mod one_of;
pub mod string;

pub use array::ArraySchema;
pub use number::NumberSchema;
pub use object::ObjectSchema;
pub use one_of::OneOfSchema;
pub use string::StringSchema;

/// A const schema represents a constant value
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ConstSchema {
    pub r#const: serde_yaml::Value,
}

/// An enum schema represents a set of constant values
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct EnumSchema {
    pub r#enum: Vec<serde_yaml::Value>,
}

#[derive(Debug, PartialEq)]
pub enum BoolOrTypedSchema {
    TypedSchema(Box<TypedSchema>),
    Boolean(bool),
}

#[derive(Debug, PartialEq)]
pub enum TypedSchema {
    Object(ObjectSchema),
    Number(NumberSchema),
    String(StringSchema),
}

/// A type value is either a string or an array of strings
#[derive(Debug, PartialEq)]
pub enum TypeValue {
    Single(serde_yaml::Value),
    Array(Vec<String>),
}

impl fmt::Display for ConstSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Const {:?}", self.r#const)
    }
}

impl From<crate::deser::ConstSchema> for ConstSchema {
    fn from(c: crate::deser::ConstSchema) -> Self {
        Self { r#const: c.r#const }
    }
}

impl fmt::Display for EnumSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Enum {:?}", self.r#enum)
    }
}

impl fmt::Display for BoolOrTypedSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoolOrTypedSchema::TypedSchema(s) => write!(f, "{}", s),
            BoolOrTypedSchema::Boolean(b) => write!(f, "{}", b),
        }
    }
}

impl fmt::Display for TypedSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypedSchema::Object(o) => write!(f, "{}", o),
            TypedSchema::Number(n) => write!(f, "{}", n),
            TypedSchema::String(s) => write!(f, "{}", s),
        }
    }
}
