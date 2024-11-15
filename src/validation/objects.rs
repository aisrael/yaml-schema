use log::debug;
/// A module to contain object type validation logic
use std::collections::HashMap;

use crate::engine::Validator;
use crate::validation::Context;
use crate::{YamlSchema, YamlSchemaError};

pub fn try_validate_value_against_properties(
    context: &Context,
    key: &String,
    value: &serde_yaml::Value,
    properties: &HashMap<String, YamlSchema>,
) -> Result<bool, YamlSchemaError> {
    let sub_context = context.append_path(key);
    if let Some(schema) = properties.get(key) {
        debug!("Validating property '{}' with schema: {}", key, schema);
        let result = schema.validate(&sub_context, value);
        match result {
            Ok(_) => return Ok(true),
            Err(e) => return Err(e),
        }
    }
    Ok(false)
}
