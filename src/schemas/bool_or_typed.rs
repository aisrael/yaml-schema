use std::fmt;

use crate::deser::Deser;
use crate::TypedSchema;
use crate::{deser, YamlSchemaError};

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

impl From<crate::deser::ArrayItemsValue> for BoolOrTypedSchema {
    fn from(value: deser::ArrayItemsValue) -> Self {
        match value {
            deser::ArrayItemsValue::Boolean(b) => BoolOrTypedSchema::Boolean(b),
            deser::ArrayItemsValue::TypedSchema(t) => {
                unimplemented!()
            }
        }
    }
}
