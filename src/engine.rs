use log::debug;

use crate::error::YamlSchemaError;
use crate::{generic_error, TypedSchema, YamlSchema};

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
        match self {
            TypedSchema::String {
                min_length,
                max_length,
                regex,
            } => {
                let yaml_string = value.as_str().ok_or_else(|| {
                    YamlSchemaError::GenericError(format!(
                        "Expected a string, but got: {:?}",
                        value
                    ))
                })?;
                if let Some(min_length) = min_length {
                    if yaml_string.len() < *min_length {
                        return generic_error!("String is too short!");
                    }
                }
                if let Some(max_length) = max_length {
                    if yaml_string.len() > *max_length {
                        return generic_error!("String is too long!");
                    }
                }
                if let Some(regex) = regex {
                    let re = regex::Regex::new(regex).map_err(|e| {
                        YamlSchemaError::GenericError(format!("Invalid regex: {}", e))
                    })?;
                    if !re.is_match(yaml_string) {
                        return generic_error!("String does not match regex!");
                    }
                }
                Ok(())
            }
            TypedSchema::Object { properties } => {
                let yaml_object = value.as_mapping().ok_or_else(|| {
                    YamlSchemaError::GenericError(format!(
                        "Expected a mapping, but got: {:?}",
                        value
                    ))
                })?;
                if let Some(properties) = properties {
                    for (property, value) in properties {
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
        }
    }
}

#[cfg(test)]
mod tests {

    // fn test_engine() {
    //     let literal = Literal::String(YamlString::with_min_length(1));
    //     let schema = YamlSchema::new();
    //     let engine = Engine::new(schema);
    //     let yaml: serde_yaml::Value = serde_yaml::from_str(r#""hello""#).unwrap();
    //     let res = engine.evaluate(&yaml);
    //     assert!(res.is_ok());

    //     let invalid_yaml = serde_yaml::from_str(r#""""#).unwrap();
    //     assert!(engine.evaluate(&invalid_yaml).is_err());
    // }
}
