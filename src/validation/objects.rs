use log::debug;
/// A module to contain object type validation logic
use std::collections::HashMap;

use crate::engine::Validator;
use crate::validation::Context;
use crate::{AdditionalProperties, TypeValue, TypedSchema, YamlSchema, YamlSchemaError};

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
    additional_properties: &AdditionalProperties,
) -> Result<bool, YamlSchemaError> {
    let sub_context = context.append_path(key);

    match additional_properties {
        // if additional_properties: true, then any additional properties are allowed
        AdditionalProperties::Boolean(true) => { /* noop */ }
        // if additional_properties: false, then no additional properties are allowed
        AdditionalProperties::Boolean(false) => {
            context.add_error(format!("Additional property '{}' is not allowed!", key));
            // returning `false` signals fail fast
            return Ok(false);
        }
        // if additional_properties: { type: <string> } or { type: [<string>] }
        // then we validate the additional property against the type schema
        AdditionalProperties::Type { r#type } => {
            // get the list of allowed types
            let allowed_types = r#type.as_list_of_allowed_types();
            debug!(
                "validate_object_mapping: allowed_types: {}",
                allowed_types.join(", ")
            );
            // TODO: Check if the value _is_ valid for any of the allow types
            // return Ok if so
            // return an error otherwise
            // check if the value is _NOT_ valid for any of the allowed types
            let allowed = allowed_types.iter().all(|allowed_type| {
                let sub_schema = TypedSchema {
                    r#type: TypeValue::from_string(allowed_type.clone()),
                    ..Default::default()
                };
                debug!(
                    "Validating additional property '{}' with schema: {:?}",
                    key, sub_schema
                );
                sub_schema.validate(&sub_context, value).is_ok()
            }); // if the value is not valid for any of the allowed types, then we return an error immediately
            if !allowed {
                context.add_error(format!(
                    "Additional property '{}' is not allowed. No allowed types matched!",
                    key
                ));
                return Ok(false);
            }
        }
    }
    Ok(true)
}
