use crate::TypedSchema;

#[derive(Debug, PartialEq)]
pub enum BoolOrTypedSchema {
    TypedSchema(Box<TypedSchema>),
    Boolean(bool),
}

impl std::fmt::Display for BoolOrTypedSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoolOrTypedSchema::TypedSchema(s) => write!(f, "{}", s),
            BoolOrTypedSchema::Boolean(b) => write!(f, "{}", b),
        }
    }
}
