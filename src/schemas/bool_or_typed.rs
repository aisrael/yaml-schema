use crate::{format_vec, TypedSchema};

#[derive(Debug, PartialEq)]
pub enum BoolOrTypedSchema {
    Boolean(bool),
    TypedSchema(Box<TypedSchema>),
    MultipleTypeNames(Vec<String>),
}

impl std::fmt::Display for BoolOrTypedSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoolOrTypedSchema::Boolean(b) => write!(f, "{}", b),
            BoolOrTypedSchema::TypedSchema(s) => write!(f, "{}", s),
            BoolOrTypedSchema::MultipleTypeNames(types) => write!(f, "{}", format_vec(types)),
        }
    }
}
