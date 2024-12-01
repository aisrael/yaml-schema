use crate::Result;
use crate::Validator;

/// A boolean schema matches any boolean value
#[derive(Debug, PartialEq)]
pub struct BooleanSchema;

impl std::fmt::Display for BooleanSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type: boolean")
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
