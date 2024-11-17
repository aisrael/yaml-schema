use log::debug;
/// A module to contain object type validation logic
use std::collections::HashMap;

use super::Validator;
use crate::schemas::{BoolOrTypedSchema, TypedSchema};
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

/// Try and validate the value against an object type's additional_properties
///
/// Returns true if the validation passed, or false if it failed (signals fail-fast)
pub fn try_validate_value_against_additional_properties(
    context: &Context,
    key: &String,
    value: &serde_yaml::Value,
    additional_properties: &BoolOrTypedSchema,
) -> Result<bool, YamlSchemaError> {
    let sub_context = context.append_path(key);

    match additional_properties {
        // if additional_properties: true, then any additional properties are allowed
        BoolOrTypedSchema::Boolean(true) => { /* noop */ }
        // if additional_properties: false, then no additional properties are allowed
        BoolOrTypedSchema::Boolean(false) => {
            context.add_error(format!("Additional property '{}' is not allowed!", key));
            // returning `false` signals fail fast
            return Ok(false);
        }
        _ => return not_yet_implemented!(),
    }
    Ok(true)
}
