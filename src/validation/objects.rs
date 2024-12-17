/// A module to contain object type validation logic
use log::debug;
use std::collections::HashMap;

use crate::schemas::BoolOrTypedSchema;
use crate::schemas::ObjectSchema;
use crate::validation::Context;
use crate::Error;
use crate::Result;
use crate::Validator;
use crate::YamlSchema;

pub fn try_validate_value_against_properties(
    context: &Context,
    key: &String,
    value: &saphyr::Yaml,
    properties: &HashMap<String, YamlSchema>,
) -> Result<bool> {
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
    value: &saphyr::Yaml,
    additional_properties: &BoolOrTypedSchema,
) -> Result<bool> {
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
        // if additional_properties: a schema, then validate against it
        BoolOrTypedSchema::TypedSchema(schema) => {
            schema.validate(&sub_context, value)?;
        }
    }
    Ok(true)
}

impl Validator for ObjectSchema {
    /// Validate the object according to the schema rules
    fn validate(&self, context: &Context, value: &saphyr::Yaml) -> Result<()> {
        debug!("Validating object: {:?}", value);
        match value {
            saphyr::Yaml::Hash(hash) => self.validate_object_mapping(context, hash),
            other => {
                context.add_error(format!("Expected an object, but got: {:#?}", other));
                Ok(())
            }
        }
    }
}

impl ObjectSchema {
    fn validate_object_mapping(&self, context: &Context, mapping: &saphyr::Hash) -> Result<()> {
        for (k, value) in mapping {
            let key = match k {
                saphyr::Yaml::String(s) => s.clone(),
                _ => k.as_str().unwrap_or_default().to_string(),
            };
            debug!("validate_object_mapping: key: \"{}\"", key);
            // First, we check the explicitly defined properties, and validate against it if found
            if let Some(properties) = &self.properties {
                if try_validate_value_against_properties(context, &key, value, properties)? {
                    continue;
                }
            }

            // Then, we check if additional properties are allowed or not
            if let Some(additional_properties) = &self.additional_properties {
                try_validate_value_against_additional_properties(
                    context,
                    &key,
                    value,
                    additional_properties,
                )?;
            }
            // Then we check if pattern_properties matches
            if let Some(pattern_properties) = &self.pattern_properties {
                for (pattern, schema) in pattern_properties {
                    // TODO: compile the regex once instead of every time we're evaluating
                    let re = regex::Regex::new(pattern).map_err(|e| {
                        Error::GenericError(format!("Invalid regular expression pattern: {}", e))
                    })?;
                    if re.is_match(key.as_str()) {
                        schema.validate(context, value)?;
                    }
                }
            }
            // Finally, we check if it matches property_names
            if let Some(property_names) = &self.property_names {
                let re = regex::Regex::new(property_names).map_err(|e| {
                    Error::GenericError(format!("Invalid regular expression pattern: {}", e))
                })?;
                debug!("Regex for property names: {}", re.as_str());
                if !re.is_match(key.as_str()) {
                    context.add_error(format!(
                        "Property name '{}' does not match pattern specified in `propertyNames`!",
                        key
                    ));
                    fail_fast!(context)
                }
            }
        }

        // Validate required properties
        if let Some(required) = &self.required {
            for required_property in required {
                let key = saphyr::Yaml::String(required_property.clone());
                if !mapping.contains_key(&key) {
                    context.add_error(format!(
                        "Required property '{}' is missing!",
                        required_property
                    ));
                    fail_fast!(context)
                }
            }
        }

        // Validate minProperties
        if let Some(min_properties) = &self.min_properties {
            if mapping.len() < *min_properties {
                context.add_error(format!(
                    "Object has too few properties! Minimum is {}!",
                    min_properties
                ));
                fail_fast!(context)
            }
        }
        // Validate maxProperties
        if let Some(max_properties) = &self.max_properties {
            if mapping.len() > *max_properties {
                context.add_error(format!(
                    "Object has too many properties! Maximum is {}!",
                    max_properties
                ));
                fail_fast!(context)
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::engine;
    use crate::NumberSchema;
    use crate::RootSchema;
    use crate::StringSchema;

    use super::*;

    #[test]
    fn test_should_validate_properties() {
        let mut properties = HashMap::new();
        properties.insert(
            "foo".to_string(),
            YamlSchema::String(StringSchema::default()),
        );
        properties.insert(
            "bar".to_string(),
            YamlSchema::Number(NumberSchema::default()),
        );
        let schema = ObjectSchema {
            properties: Some(properties),
            ..Default::default()
        };
        let yaml_schema = YamlSchema::Object(schema);
        let root_schema = RootSchema::new(yaml_schema);
        let value = r#"
            foo: "I'm a string"
            bar: 42
        "#;
        let result = engine::Engine::evaluate(&root_schema, value, true);
        assert!(result.is_ok());

        let value2 = r#"
            foo: 42
            baz: "I'm a string"
        "#;
        let context = engine::Engine::evaluate(&root_schema, value2, true).unwrap();
        assert!(context.has_errors());
        let errors = context.errors.borrow();
        let first_error = errors.first().unwrap();
        assert_eq!(first_error.path, "foo");
        assert_eq!(first_error.error, "Expected a string, but got: Integer(42)");
    }
}
