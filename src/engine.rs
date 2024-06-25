use log::debug;

use crate::error::YamlSchemaError;
use crate::{generic_error, not_yet_implemented, TypeValue, TypedSchema, YamlSchema};

pub struct Engine<'a> {
    pub schema: &'a YamlSchema,
}

impl<'a> Engine<'a> {
    pub fn new(schema: &'a YamlSchema) -> Engine<'a> {
        Engine { schema }
    }

    pub fn evaluate(&self, yaml: &serde_yaml::Value) -> Result<(), YamlSchemaError> {
        debug!("Engine is running");
        self.schema.validate(yaml)
    }
}

pub trait Validator {
    fn validate(&self, value: &serde_yaml::Value) -> Result<(), YamlSchemaError>;
}

impl Validator for YamlSchema {
    fn validate(&self, value: &serde_yaml::Value) -> Result<(), YamlSchemaError> {
        debug!("self: {:?}", self);
        debug!("Validating value: {:?}", value);
        match self {
            YamlSchema::Empty => Ok(()),
            YamlSchema::Boolean(boolean) => {
                if *boolean {
                    Ok(())
                } else {
                    generic_error!("Schema is `false`!")
                }
            }
            YamlSchema::TypedSchema(typed_schema) => {
                debug!("Schema value: {:?}", typed_schema);
                typed_schema.validate(value)
            }
        }
    }
}

impl Validator for TypedSchema {
    fn validate(&self, value: &serde_yaml::Value) -> Result<(), YamlSchemaError> {
        debug!("Validating value: {:?}", value);

        match self.r#type {
            TypeValue::String(ref s) => match s.as_str() {
                "string" => {
                    let yaml_string = value.as_str().ok_or_else(|| {
                        YamlSchemaError::GenericError(format!(
                            "Expected a string, but got: {:?}",
                            value
                        ))
                    })?;
                    if let Some(min_length) = &self.min_length {
                        if yaml_string.len() < *min_length {
                            return generic_error!("String is too short!");
                        }
                    }
                    if let Some(max_length) = &self.max_length {
                        if yaml_string.len() > *max_length {
                            return generic_error!("String is too long!");
                        }
                    }
                    if let Some(regex) = &self.regex {
                        let re = regex::Regex::new(regex).map_err(|e| {
                            YamlSchemaError::GenericError(format!("Invalid regex: {}", e))
                        })?;
                        if !re.is_match(yaml_string) {
                            return generic_error!("String does not match regex!");
                        }
                    }
                    Ok(())
                }
                "object" => {
                    let yaml_object = value.as_mapping().ok_or_else(|| {
                        YamlSchemaError::GenericError(format!(
                            "Expected a mapping, but got: {:?}",
                            value
                        ))
                    })?;
                    if let Some(properties) = &self.properties {
                        for property in properties.keys() {
                            if !yaml_object.contains_key(&serde_yaml::Value::String(property.clone())) {
                                return Err(YamlSchemaError::GenericError(format!(
                                    "Property `{}` is missing!",
                                    property
                                )));
                            }
                        }
                    }
                    Ok(())
                }
                _ => not_yet_implemented!(),
            },
            TypeValue::Array(_) => {
                not_yet_implemented!()
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_properties_with_no_value() {
        let schema = TypedSchema::object(
            vec![
                ("name".to_string(), YamlSchema::Empty),
                ("age".to_string(), YamlSchema::Empty),
            ]
            .into_iter()
            .collect(),
        );
        let yaml_schema = YamlSchema::TypedSchema(schema);
        let engine = Engine::new(&yaml_schema);
        let yaml = serde_yaml::from_str(
            r#"
            name: "John Doe"
            age: 42
        "#,
        )
        .unwrap();
        assert!(engine.evaluate(&yaml).is_ok());
    }
}
