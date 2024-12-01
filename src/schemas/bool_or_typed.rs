use std::fmt;

use crate::TypedSchema;

#[derive(Debug, PartialEq)]
pub enum BoolOrTypedSchema {
    TypedSchema(Box<TypedSchema>),
    Boolean(bool),
}

impl fmt::Display for BoolOrTypedSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoolOrTypedSchema::TypedSchema(s) => write!(f, "{}", s),
            BoolOrTypedSchema::Boolean(b) => write!(f, "{}", b),
        }
    }
}
