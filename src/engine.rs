use log::debug;

use crate::error::YamlSchemaError;
use crate::{
    generic_error, not_yet_implemented, EnumSchema, TypeValue, TypedSchema, YamlSchema,
    YamlSchemaNumber,
};

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
            YamlSchema::Enum(enum_schema) => enum_schema.validate(value),
        }
    }
}

impl Validator for TypedSchema {
    fn validate(&self, value: &serde_yaml::Value) -> Result<(), YamlSchemaError> {
        debug!("Validating value: {:?}", value);

        match self.r#type {
            TypeValue::String(ref s) => match s.as_str() {
                "integer" => self.validate_integer(value),
                "number" => self.validate_number(value),
                "object" => self.validate_object(value),
                "string" => self.validate_string(value),
                _ => generic_error!("Unknown type '{}'!", s),
            },
            TypeValue::Array(_) => {
                not_yet_implemented!()
            }
        }
    }
}

impl TypedSchema {
    fn validate_integer(&self, value: &serde_yaml::Value) -> Result<(), YamlSchemaError> {
        if !value.is_i64() {
            if value.is_f64() {
                let f = value.as_f64().unwrap();
                if f.fract() == 0.0 {
                    return self.validate_number_i64(f as i64);
                } else {
                    return generic_error!("Expected an integer, but got: {:?}", value);
                }
            }
            return generic_error!("Expected an integer, but got: {:?}", value);
        }
        let i = value.as_i64().unwrap();
        self.validate_number_i64(i)
    }

    fn validate_number(&self, value: &serde_yaml::Value) -> Result<(), YamlSchemaError> {
        if value.is_i64() {
            match value.as_i64() {
                Some(i) => self.validate_number_i64(i),
                None => generic_error!("Expected an integer, but got: {:?}", value),
            }
        } else if value.is_f64() {
            match value.as_f64() {
                Some(f) => self.validate_number_f64(f),
                None => generic_error!("Expected a float, but got: {:?}", value),
            }
        } else {
            return generic_error!("Expected a number, but got: {:?}", value);
        }
    }

    fn validate_number_i64(&self, i: i64) -> Result<(), YamlSchemaError> {
        if let Some(minimum) = &self.minimum {
            match minimum {
                YamlSchemaNumber::Integer(min) => {
                    if i < *min {
                        return generic_error!("Number is too small!");
                    }
                }
                YamlSchemaNumber::Float(min) => {
                    if (i as f64) < *min {
                        return generic_error!("Number is too small!");
                    }
                }
            }
        }
        if let Some(maximum) = &self.maximum {
            match maximum {
                YamlSchemaNumber::Integer(max) => {
                    if i > *max {
                        return generic_error!("Number is too big!");
                    }
                }
                YamlSchemaNumber::Float(max) => {
                    if (i as f64) > *max {
                        return generic_error!("Number is too big!");
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_number_f64(&self, f: f64) -> Result<(), YamlSchemaError> {
        if let Some(minimum) = &self.minimum {
            match minimum {
                YamlSchemaNumber::Integer(min) => {
                    if f < *min as f64 {
                        return generic_error!("Number is too small!");
                    }
                }
                YamlSchemaNumber::Float(min) => {
                    if f < *min {
                        return generic_error!("Number is too small!");
                    }
                }
            }
        }
        if let Some(maximum) = &self.maximum {
            match maximum {
                YamlSchemaNumber::Integer(max) => {
                    if f > *max as f64 {
                        return generic_error!("Number is too big!");
                    }
                }
                YamlSchemaNumber::Float(max) => {
                    if f > *max {
                        return generic_error!("Number is too big!");
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_string(&self, value: &serde_yaml::Value) -> Result<(), YamlSchemaError> {
        let yaml_string = value.as_str().ok_or_else(|| {
            YamlSchemaError::GenericError(format!("Expected a string, but got: {:?}", value))
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
        if let Some(pattern) = &self.pattern {
            let re = regex::Regex::new(pattern).map_err(|e| {
                YamlSchemaError::GenericError(format!("Invalid regular expression pattern: {}", e))
            })?;
            if !re.is_match(yaml_string) {
                return generic_error!("String does not match regex!");
            }
        }
        Ok(())
    }

    fn validate_object(&self, value: &serde_yaml::Value) -> Result<(), YamlSchemaError> {
        let yaml_object = value.as_mapping().ok_or_else(|| {
            YamlSchemaError::GenericError(format!("Expected a mapping, but got: {:?}", value))
        })?;
        if let Some(properties) = &self.properties {
            for (property, schema) in properties {
                let key = &serde_yaml::Value::String(property.clone());
                if yaml_object.contains_key(key) {
                    schema.validate(&yaml_object[key])?;
                }
            }
        }
        if let Some(required_properties) = &self.required {
            for required_property in required_properties {
                let key = &serde_yaml::Value::String(required_property.clone());
                if !yaml_object.contains_key(key) {
                    return Err(YamlSchemaError::GenericError(format!(
                        "Required property '{}' is missing!",
                        required_property
                    )));
                }
            }
        }
        Ok(())
    }
}

impl Validator for EnumSchema {
    fn validate(&self, value: &serde_yaml::Value) -> Result<(), YamlSchemaError> {
        if !self.r#enum.contains(value) {
            return generic_error!("Value is not in the enum!");
        }
        Ok(())
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
        let yaml_schema = YamlSchema::TypedSchema(Box::new(schema));
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

    #[test]
    fn test_leaving_out_properties_is_valid() {
        let object_schema = TypedSchema::object(
            vec![
                (
                    "number".to_string(),
                    YamlSchema::TypedSchema(Box::new(TypedSchema::number())),
                ),
                (
                    "street_name".to_string(),
                    YamlSchema::TypedSchema(Box::new(TypedSchema::string())),
                ),
                (
                    "street_type".to_string(),
                    YamlSchema::Enum(EnumSchema::new(vec![
                        "Street".to_string(),
                        "Avenue".to_string(),
                        "Boulevard".to_string(),
                    ])),
                ),
            ]
            .into_iter()
            .collect(),
        );
        let yaml_schema = YamlSchema::TypedSchema(Box::new(object_schema));
        let engine = Engine::new(&yaml_schema);
        let yaml = serde_yaml::from_str(
            r#"
            number: 1600
            street_name: Pennsylvania
        "#,
        )
        .unwrap();
        let result = engine.evaluate(&yaml);
        if let Err(e) = result {
            panic!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }
}
