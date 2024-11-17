use std::fmt;

/// An enum schema represents a set of constant values
#[derive(Debug, PartialEq)]
pub struct EnumSchema {
    pub r#enum: Vec<serde_yaml::Value>,
}

impl fmt::Display for EnumSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Enum {:?}", self.r#enum)
    }
}

impl From<&crate::deser::EnumSchema> for EnumSchema {
    fn from(e: &crate::deser::EnumSchema) -> Self {
        Self {
            r#enum: e.r#enum.clone(),
        }
    }
}
