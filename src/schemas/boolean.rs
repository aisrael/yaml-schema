use std::fmt::{self};

use crate::Result;
use crate::Validator;

/// A boolean schema matches any boolean value
#[derive(Debug, PartialEq)]
pub struct BooleanSchema;

impl fmt::Display for BooleanSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type: boolean")
    }
}

impl From<&crate::deser::TypedSchema> for BooleanSchema {
    fn from(t: &crate::deser::TypedSchema) -> Self {
        if t.r#type
            == crate::deser::TypeValue::Single(serde_yaml::Value::String("boolean".to_string()))
        {
            BooleanSchema {}
        } else {
            panic!("Expected type: boolean")
        }
    }
}

impl Validator for BooleanSchema {
    fn validate(
        &self,
        context: &crate::validation::Context,
        value: &serde_yaml::Value,
    ) -> Result<()> {
        if !value.is_bool() {
            context.add_error(format!("Expected: boolean, found: {:?}", value));
        }
        Ok(())
    }
}
