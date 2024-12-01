use std::fmt;

/// A const schema represents a constant value
#[derive(Debug, PartialEq)]
pub struct ConstSchema {
    pub r#const: serde_yaml::Value,
}

impl fmt::Display for ConstSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Const {:?}", self.r#const)
    }
}
