use crate::TypedSchema;

#[derive(Debug, PartialEq)]
pub enum BoolOrTypedSchema {
    Boolean(bool),
    TypedSchema(Box<TypedSchema>),
}

impl std::fmt::Display for BoolOrTypedSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoolOrTypedSchema::Boolean(b) => write!(f, "{}", b),
            BoolOrTypedSchema::TypedSchema(s) => write!(f, "{}", s),
        }
    }
}
