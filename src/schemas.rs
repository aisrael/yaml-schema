/// The schemas defined in the YAML schema language
use std::fmt;

mod any_of;
mod array;
mod bool_or_typed;
mod boolean;
mod r#const;
mod r#enum;
mod number;
mod object;
mod one_of;
mod string;

pub use array::ArraySchema;
pub use bool_or_typed::BoolOrTypedSchema;
pub use boolean::BooleanSchema;
pub use number::NumberSchema;
pub use object::ObjectSchema;
pub use one_of::OneOfSchema;
pub use r#const::ConstSchema;
pub use r#enum::EnumSchema;
pub use string::StringSchema;

#[derive(Debug, PartialEq)]
pub enum TypedSchema {
    Array(ArraySchema),
    Boolean,
    Empty,
    Number(NumberSchema),
    Object(ObjectSchema),
    String(StringSchema),
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
            TypedSchema::Empty => write!(f, "type: null"),
            TypedSchema::Number(n) => write!(f, "{}", n),
            TypedSchema::Object(o) => write!(f, "{}", o),
            TypedSchema::String(s) => write!(f, "{}", s),
        }
    }
}
