use log::debug;
use std::collections::HashMap;
use std::fmt;

use super::BoolOrTypedSchema;
use crate::validation::objects::try_validate_value_against_additional_properties;
use crate::validation::objects::try_validate_value_against_properties;
use crate::Result;
use crate::{Context, Error, PropertyNamesValue, Validator, YamlSchema};

/// An object schema
#[derive(Debug, Default, PartialEq)]
pub struct ObjectSchema {
    pub properties: Option<HashMap<String, YamlSchema>>,
    pub required: Option<Vec<String>>,
    pub additional_properties: Option<BoolOrTypedSchema>,
    pub pattern_properties: Option<HashMap<String, YamlSchema>>,
    pub property_names: Option<PropertyNamesValue>,
    pub min_properties: Option<usize>,
    pub max_properties: Option<usize>,
}

impl fmt::Display for ObjectSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Object {:?}", self)
    }
}

impl Validator for ObjectSchema {
    /// Validate the object according to the schema rules
    fn validate(&self, context: &Context, value: &serde_yaml::Value) -> Result<()> {
        debug!("Validating object: {:?}", value);
        match value.as_mapping() {
            Some(mapping) => self.validate_object_mapping(context, mapping),
            None => {
                context.add_error("Expected an object, but got: None");
                Ok(())
            }
        }
    }
}

impl ObjectSchema {
    fn validate_object_mapping(
        &self,
        context: &Context,
        mapping: &serde_yaml::Mapping,
    ) -> Result<()> {
        for (k, value) in mapping {
            let key = match k {
                serde_yaml::Value::String(s) => s.clone(),
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
                let re = regex::Regex::new(&property_names.pattern).map_err(|e| {
                    Error::GenericError(format!("Invalid regular expression pattern: {}", e))
                })?;
                debug!("Regex for property names: {}", re.as_str());
                if !re.is_match(key.as_str()) {
                    return Err(Error::GenericError(format!(
                        "Property name '{}' does not match pattern specified in `propertyNames`!",
                        key
                    )));
                }
            }
        }

        // Validate required properties
        if let Some(required) = &self.required {
            for required_property in required {
                if !mapping.contains_key(required_property) {
                    return Err(Error::GenericError(format!(
                        "Required property '{}' is missing!",
                        required_property
                    )));
                }
            }
        }

        // Validate minProperties
        if let Some(min_properties) = &self.min_properties {
            if mapping.len() < *min_properties {
                return Err(Error::GenericError(format!(
                    "Object has too few properties! Minimum is {}!",
                    min_properties
                )));
            }
        }
        // Validate maxProperties
        if let Some(max_properties) = &self.max_properties {
            if mapping.len() > *max_properties {
                return Err(Error::GenericError(format!(
                    "Object has too many properties! Maximum is {}!",
                    max_properties
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::engine;
    use crate::NumberSchema;
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
        let engine = engine::Engine::new(&yaml_schema);
        let value = serde_yaml::from_str(
            r#"
            foo: "I'm a string"
            bar: 42
        "#,
        )
        .unwrap();
        assert!(engine.evaluate(&value, true).is_ok());

        let value = serde_yaml::from_str(
            r#"
            foo: 42
            baz: "I'm a string"
        "#,
        )
        .unwrap();
        let context = engine.evaluate(&value, true).unwrap();
        assert!(context.has_errors());
        let errors = context.errors.borrow();
        let first_error = errors.first().unwrap();
        assert_eq!(first_error.path, "foo");
        assert_eq!(first_error.error, "Expected a string, but got: Number(42)");
    }
}
