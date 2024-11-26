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

impl From<&crate::deser::ConstSchema> for ConstSchema {
    fn from(c: &crate::deser::ConstSchema) -> Self {
        Self {
            r#const: c.r#const.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deser;

    #[test]
    fn test_from_deser_const_schema() {
        let deser_schema = deser::ConstSchema {
            r#const: serde_yaml::Value::String("test".to_string()),
        };

        let schema: ConstSchema = (&deser_schema).into();

        assert_eq!(schema.r#const, deser_schema.r#const);
    }
}
