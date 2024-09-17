use log::debug;

use crate::error::YamlSchemaError;
use crate::{
    generic_error, not_yet_implemented, AdditionalProperties, EnumSchema, TypeValue, TypedSchema,
    YamlSchema, YamlSchemaNumber,
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
        debug!("self: {}", self);
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
                debug!("Schema value: {}", typed_schema);
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

    /// Validate the string according to the schema rules
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

    /// Validate the object according to the schema rules
    fn validate_object(&self, value: &serde_yaml::Value) -> Result<(), YamlSchemaError> {
        let mapping = value.as_mapping().ok_or_else(|| {
            YamlSchemaError::GenericError(format!("Expected a mapping, but got: {:?}", value))
        })?;

        for (k, value) in mapping {
            let key = match k {
                serde_yaml::Value::String(s) => s.clone(),
                _ => k.as_str().unwrap_or_default().to_string(),
            };
            // First, we check the explicitly defined properties, and validate against it if found
            if let Some(properties) = &self.properties {
                if properties.contains_key(&key) {
                    match properties[&key].validate(value) {
                        Err(e) => return Err(e),
                        Ok(_) => continue,
                    }
                }
            }
            // Then, we check if additional properties are allowed or not
            if let Some(additional_properties) = &self.additional_properties {
                match additional_properties {
                    // if additional_properties: true, then any additional properties are allowed
                    AdditionalProperties::Boolean(true) => { /* no-op */ }
                    // if additional_properties: false, then no additional properties are allowed
                    AdditionalProperties::Boolean(false) => {
                        return Err(YamlSchemaError::GenericError(format!(
                            "Additional property '{}' is not allowed!",
                            key
                        )));
                    }
                    // if additional_properties: { type: <string> } or { type: [<string>] }
                    // then we validate the additional property against the type schema
                    AdditionalProperties::Type { r#type } => {
                        // get the list of allowed types
                        let allowed_types = r#type.as_list_of_allowed_types();
                        // check if the value is _NOT_ valid for any of the allowed types
                        let is_invalid = allowed_types.iter().any(|allowed_type| {
                            let typed_schema = TypedSchema {
                                r#type: TypeValue::String(allowed_type.clone()),
                                ..Default::default()
                            };
                            debug!(
                                "Validating additional property '{}' with schema: {:?}",
                                key, typed_schema
                            );
                            let res = typed_schema.validate(value);
                            res.is_err()
                        });
                        // if the value is not valid for any of the allowed types, then we return an error immediately
                        if is_invalid {
                            return Err(YamlSchemaError::GenericError(format!(
                                "Additional property '{}' is not allowed. No allowed types matched!",
                                key
                            )));
                        }
                    }
                }
            }
            if let Some(pattern_properties) = &self.pattern_properties {
                for (pattern, schema) in pattern_properties {
                    // TODO: compile the regex once instead of every time we're evaluating
                    let re = regex::Regex::new(pattern).map_err(|e| {
                        YamlSchemaError::GenericError(format!(
                            "Invalid regular expression pattern: {}",
                            e
                        ))
                    })?;
                    if re.is_match(key.as_str()) {
                        schema.validate(value)?
                    }
                }
            }
        }

        // Validate required properties
        if let Some(required) = &self.required {
            for required_property in required {
                if !mapping.contains_key(required_property) {
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
    fn test_additional_properties_are_valid() {
        let additional_properties = AdditionalProperties::Type {
            r#type: TypeValue::String("string".to_string()),
        };
        let schema = TypedSchema {
            additional_properties: Some(additional_properties),
            ..Default::default()
        };
        let yaml_schema = YamlSchema::typed_schema(schema);
        let engine = Engine::new(&yaml_schema);
        let yaml = serde_yaml::from_str(
            r#"
            name: "John Doe"
        "#,
        )
        .unwrap();
        assert!(engine.evaluate(&yaml).is_ok());

        let invalid_yaml = serde_yaml::from_str(
            r#"
            age: 42
        "#,
        )
        .unwrap();
        let invalid_result = engine.evaluate(&invalid_yaml);
        assert!(invalid_result.is_err());
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
