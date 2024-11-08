use log::debug;

use crate::error::YamlSchemaError;
use crate::{
    format_vec, generic_error, not_yet_implemented, AdditionalProperties, ArrayItemsValue,
    ConstSchema, Context, EnumSchema, OneOfSchema, TypeValue, TypedSchema, YamlSchema,
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
        let mut context = Context::new(self.schema);
        self.schema.validate(&mut context, yaml)
    }
}

pub trait Validator {
    fn validate(
        &self,
        context: &mut Context,
        value: &serde_yaml::Value,
    ) -> Result<(), YamlSchemaError>;
}

impl Validator for YamlSchema {
    fn validate(
        &self,
        context: &mut Context,
        value: &serde_yaml::Value,
    ) -> Result<(), YamlSchemaError> {
        debug!("self: {}", self);
        debug!("Validating value: {:?}", value);
        match self {
            YamlSchema::Empty => Ok(()),
            YamlSchema::Boolean(boolean) => {
                if *boolean {
                    Ok(())
                } else {
                    Err(generic_error!("Schema is `false`!"))
                }
            }
            YamlSchema::Const(const_schema) => const_schema.validate(context, value),
            YamlSchema::TypedSchema(typed_schema) => {
                debug!("Schema value: {}", typed_schema);
                typed_schema.validate(context, value)
            }
            YamlSchema::Enum(enum_schema) => enum_schema.validate(context, value),
            YamlSchema::OneOf(one_of_schema) => one_of_schema.validate(context, value),
        }
    }
}

impl Validator for TypedSchema {
    fn validate(
        &self,
        context: &mut Context,
        value: &serde_yaml::Value,
    ) -> Result<(), YamlSchemaError> {
        debug!("Validating value: {:?}", value);

        match self.r#type {
            TypeValue::Single(ref v) => match v {
                serde_yaml::Value::String(ref s) => match s.as_str() {
                    "array" => self.validate_array(context, value),
                    "boolean" => self.validate_boolean(value),
                    "integer" => self.validate_integer(value),
                    "number" => self.validate_number(value),
                    "object" => self.validate_object(context, value),
                    "string" => self.validate_string(value),
                    _ => Err(generic_error!("Unknown type '{}'!", s)),
                },
                serde_yaml::Value::Null => {
                    if !value.is_null() {
                        return Err(generic_error!("Expected a value, but got: {:?}", value));
                    }
                    Ok(())
                }
                _ => Err(generic_error!("Expected a string, but got: {:?}", value)),
            },
            TypeValue::Array(ref _types) => {
                not_yet_implemented!()
            }
        }
    }
}

impl TypedSchema {
    fn validate_boolean(&self, value: &serde_yaml::Value) -> Result<(), YamlSchemaError> {
        if !value.is_bool() {
            return Err(generic_error!("Expected a boolean, but got: {:?}", value));
        }
        Ok(())
    }

    fn validate_integer(&self, value: &serde_yaml::Value) -> Result<(), YamlSchemaError> {
        if !value.is_i64() {
            if value.is_f64() {
                let f = value.as_f64().unwrap();
                if f.fract() == 0.0 {
                    return self.validate_number_i64(f as i64);
                } else {
                    return Err(generic_error!("Expected an integer, but got: {:?}", value));
                }
            }
            return Err(generic_error!("Expected an integer, but got: {:?}", value));
        }
        let i = value.as_i64().unwrap();
        self.validate_number_i64(i)
    }

    fn validate_number(&self, value: &serde_yaml::Value) -> Result<(), YamlSchemaError> {
        if value.is_i64() {
            match value.as_i64() {
                Some(i) => self.validate_number_i64(i),
                None => Err(generic_error!("Expected an integer, but got: {:?}", value)),
            }
        } else if value.is_f64() {
            match value.as_f64() {
                Some(f) => self.validate_number_f64(f),
                None => Err(generic_error!("Expected a float, but got: {:?}", value)),
            }
        } else {
            return Err(generic_error!("Expected a number, but got: {:?}", value));
        }
    }

    fn validate_number_i64(&self, i: i64) -> Result<(), YamlSchemaError> {
        if let Some(minimum) = &self.minimum {
            match minimum {
                YamlSchemaNumber::Integer(min) => {
                    if i < *min {
                        return Err(generic_error!("Number is too small!"));
                    }
                }
                YamlSchemaNumber::Float(min) => {
                    if (i as f64) < *min {
                        return Err(generic_error!("Number is too small!"));
                    }
                }
            }
        }
        if let Some(maximum) = &self.maximum {
            match maximum {
                YamlSchemaNumber::Integer(max) => {
                    if i > *max {
                        return Err(generic_error!("Number is too big!"));
                    }
                }
                YamlSchemaNumber::Float(max) => {
                    if (i as f64) > *max {
                        return Err(generic_error!("Number is too big!"));
                    }
                }
            }
        }
        if let Some(multiple_of) = &self.multiple_of {
            match multiple_of {
                YamlSchemaNumber::Integer(multiple) => {
                    if i % *multiple != 0 {
                        return Err(generic_error!("Number is not a multiple of {}!", multiple));
                    }
                }
                YamlSchemaNumber::Float(multiple) => {
                    if (i as f64) % *multiple != 0.0 {
                        return Err(generic_error!("Number is not a multiple of {}!", multiple));
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
                        return Err(generic_error!("Number is too small!"));
                    }
                }
                YamlSchemaNumber::Float(min) => {
                    if f < *min {
                        return Err(generic_error!("Number is too small!"));
                    }
                }
            }
        }
        if let Some(maximum) = &self.maximum {
            match maximum {
                YamlSchemaNumber::Integer(max) => {
                    if f > *max as f64 {
                        return Err(generic_error!("Number is too big!"));
                    }
                }
                YamlSchemaNumber::Float(max) => {
                    if f > *max {
                        return Err(generic_error!("Number is too big!"));
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
                return Err(generic_error!("String is too short!"));
            }
        }
        if let Some(max_length) = &self.max_length {
            if yaml_string.len() > *max_length {
                return Err(generic_error!("String is too long!"));
            }
        }
        if let Some(pattern) = &self.pattern {
            let re = regex::Regex::new(pattern).map_err(|e| {
                YamlSchemaError::GenericError(format!("Invalid regular expression pattern: {}", e))
            })?;
            if !re.is_match(yaml_string) {
                return Err(generic_error!("String does not match regex!"));
            }
        }
        Ok(())
    }

    /// Validate the object according to the schema rules
    fn validate_object(
        &self,
        context: &mut Context,
        value: &serde_yaml::Value,
    ) -> Result<(), YamlSchemaError> {
        let mapping = value.as_mapping().ok_or_else(|| {
            YamlSchemaError::GenericError(format!(
                "Expected an object, but got: {}",
                format_serde_yaml_value(value)
            ))
        })?;

        for (k, value) in mapping {
            let key = match k {
                serde_yaml::Value::String(s) => s.clone(),
                _ => k.as_str().unwrap_or_default().to_string(),
            };
            // First, we check the explicitly defined properties, and validate against it if found
            if let Some(properties) = &self.properties {
                if properties.contains_key(&key) {
                    match properties[&key].validate(context, value) {
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
                                r#type: TypeValue::from_string(allowed_type.clone()),
                                ..Default::default()
                            };
                            debug!(
                                "Validating additional property '{}' with schema: {:?}",
                                key, typed_schema
                            );
                            let res = typed_schema.validate(context, value);
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
                        schema.validate(context, value)?
                    }
                }
            }
            if let Some(property_names) = &self.property_names {
                let re = regex::Regex::new(&property_names.pattern).map_err(|e| {
                    YamlSchemaError::GenericError(format!(
                        "Invalid regular expression pattern: {}",
                        e
                    ))
                })?;
                debug!("Regex for property names: {}", re.as_str());
                if !re.is_match(key.as_str()) {
                    return Err(YamlSchemaError::GenericError(format!(
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
                    return Err(YamlSchemaError::GenericError(format!(
                        "Required property '{}' is missing!",
                        required_property
                    )));
                }
            }
        }

        // Validate minProperties
        if let Some(min_properties) = &self.min_properties {
            if mapping.len() < *min_properties {
                return Err(YamlSchemaError::GenericError(format!(
                    "Object has too few properties! Minimum is {}!",
                    min_properties
                )));
            }
        }
        // Validate maxProperties
        if let Some(max_properties) = &self.max_properties {
            if mapping.len() > *max_properties {
                return Err(YamlSchemaError::GenericError(format!(
                    "Object has too many properties! Maximum is {}!",
                    max_properties
                )));
            }
        }

        Ok(())
    }

    fn validate_array(
        &self,
        context: &mut Context,
        value: &serde_yaml::Value,
    ) -> Result<(), YamlSchemaError> {
        if !value.is_sequence() {
            return Err(generic_error!("Expected an array, but got: {:?}", value));
        }

        let array = value.as_sequence().unwrap();

        // validate array items
        if let Some(items) = &self.items {
            match items {
                ArrayItemsValue::TypedSchema(typed_schema) => {
                    for item in array {
                        typed_schema.validate(context, item)?;
                    }
                }
                ArrayItemsValue::Boolean(true) => { /* no-op */ }
                ArrayItemsValue::Boolean(false) => {
                    if self.prefix_items.is_none() {
                        return Err(YamlSchemaError::GenericError(
                            "Array items are not allowed!".to_string(),
                        ));
                    }
                }
            }
        }

        // validate contains
        if let Some(contains) = &self.contains {
            if !array
                .iter()
                .any(|item| contains.validate(context, item).is_ok())
            {
                return Err(YamlSchemaError::GenericError(
                    "Contains validation failed!".to_string(),
                ));
            }
        }

        // validate prefix items
        if let Some(prefix_items) = &self.prefix_items {
            debug!("Validating prefix items: {}", format_vec(prefix_items));
            for (i, item) in array.iter().enumerate() {
                // if the index is within the prefix items, validate against the prefix items schema
                if i < prefix_items.len() {
                    debug!(
                        "Validating prefix item {} with schema: {}",
                        i, prefix_items[i]
                    );
                    prefix_items[i].validate(context, item)?;
                } else if let Some(items) = &self.items {
                    // if the index is not within the prefix items, validate against the array items schema
                    match items {
                        ArrayItemsValue::TypedSchema(typed_schema) => {
                            typed_schema.validate(context, item)?;
                        }
                        ArrayItemsValue::Boolean(true) => {
                            // `items: true` allows any items
                            break;
                        }
                        ArrayItemsValue::Boolean(false) => {
                            return Err(YamlSchemaError::GenericError(
                                "Additional array items are not allowed!".to_string(),
                            ));
                        }
                    }
                } else {
                    break;
                }
            }
        }

        Ok(())
    }
}

impl Validator for ConstSchema {
    fn validate(
        &self,
        context: &mut Context,
        value: &serde_yaml::Value,
    ) -> Result<(), YamlSchemaError> {
        debug!(
            "Validating value: {:?} against const: {:?}",
            value, self.r#const
        );
        let expected_value = &self.r#const;
        if expected_value != value {
            let error = generic_error!(
                "Const validation failed, expected: {:?}, got: {:?}",
                expected_value,
                value
            );
            context.add_error(error.clone());
            return Err(error);
        }
        Ok(())
    }
}

fn format_serde_yaml_value(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::Null => "null".to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::String(s) => format!("\"{}\"", s),
        _ => format!("{:?}", value),
    }
}

impl Validator for EnumSchema {
    fn validate(
        &self,
        context: &mut Context,
        value: &serde_yaml::Value,
    ) -> Result<(), YamlSchemaError> {
        if !self.r#enum.contains(value) {
            let value_str = format_serde_yaml_value(value);
            let enum_values = self
                .r#enum
                .iter()
                .map(format_serde_yaml_value)
                .collect::<Vec<String>>()
                .join(", ");
            let error = generic_error!("Value {} is not in the enum: [{}]", value_str, enum_values);
            context.add_error(error.clone());
            return Err(error);
        }
        Ok(())
    }
}

impl Validator for OneOfSchema {
    fn validate(
        &self,
        context: &mut Context,
        value: &serde_yaml::Value,
    ) -> Result<(), YamlSchemaError> {
        {
            let schemas: &Vec<YamlSchema> = &self.one_of;
            let mut one_of_is_valid = false;
            for schema in schemas {
                debug!("Validating value: {:?} against schema: {}", value, schema);
                if schema.validate(context, value).is_ok() {
                    if one_of_is_valid {
                        return Err(generic_error!("Value matched multiple schemas in `oneOf`!"));
                    }
                    one_of_is_valid = true;
                }
            }
            if one_of_is_valid {
                Ok(())
            } else {
                Err(generic_error!("None of the schemas in `oneOf` matched!"))
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_const() {
        let const_schema = ConstSchema::new("United States of America");
        let yaml_schema = YamlSchema::Const(const_schema);
        let mut context = Context::new(&yaml_schema);
        assert!(yaml_schema
            .validate(
                &mut context,
                &serde_yaml::Value::String("United States of America".to_string())
            )
            .is_ok());
        assert!(yaml_schema
            .validate(
                &mut context,
                &serde_yaml::Value::String("Canada".to_string())
            )
            .is_err());
        let schema = TypedSchema::object(
            vec![(
                "country".to_string(),
                YamlSchema::const_schema("United States of America"),
            )]
            .into_iter()
            .collect(),
        );
        let yaml_schema = YamlSchema::TypedSchema(Box::new(schema));
        let yaml = serde_yaml::from_str(
            r#"
            country: United States of America
        "#,
        )
        .unwrap();
        let result = yaml_schema.validate(&mut context, &yaml);
        assert!(result.is_ok());

        let parsed_yaml_schema: YamlSchema = serde_yaml::from_str(
            r#"
            const: United States of America
            "#,
        )
        .unwrap();
        let result = parsed_yaml_schema.validate(
            &mut context,
            &serde_yaml::Value::String("United States of America".to_string()),
        );
        assert!(result.is_ok());
    }

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
            r#type: TypeValue::string(),
        };
        let schema = TypedSchema {
            r#type: TypeValue::object(),
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

    #[test]
    fn test_one_of_with_multiple_schemas() {
        let one_of_schema = OneOfSchema {
            one_of: vec![
                YamlSchema::TypedSchema(Box::new(TypedSchema {
                    r#type: TypeValue::number(),
                    multiple_of: Some(YamlSchemaNumber::Integer(5)),
                    ..Default::default()
                })),
                YamlSchema::TypedSchema(Box::new(TypedSchema {
                    r#type: TypeValue::number(),
                    multiple_of: Some(YamlSchemaNumber::Integer(3)),
                    ..Default::default()
                })),
            ],
        };
        let yaml = serde_yaml::from_str(
            r#"
            10
        "#,
        )
        .unwrap();
        let yaml_schema = YamlSchema::OneOf(one_of_schema);
        let mut context = Context::new(&yaml_schema);
        let result = yaml_schema.validate(&mut context, &yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pattern_properties_with_one_of() {
        let one_of: Vec<YamlSchema> = vec![
            YamlSchema::TypedSchema(Box::new(TypedSchema::null())),
            YamlSchema::TypedSchema(Box::new(TypedSchema::object(
                vec![(
                    "name".to_string(),
                    YamlSchema::TypedSchema(Box::new(TypedSchema {
                        r#type: TypeValue::string(),
                        additional_properties: Some(AdditionalProperties::Boolean(false)),
                        ..Default::default()
                    })),
                )]
                .into_iter()
                .collect(),
            ))),
        ];
        let pattern_properties: HashMap<String, YamlSchema> = HashMap::from([(
            "^[a-zA-Z0-9]+$".to_string(),
            YamlSchema::OneOf(OneOfSchema { one_of }),
        )]);
        let pattern_properties_schema: TypedSchema = TypedSchema {
            r#type: TypeValue::object(),
            pattern_properties: Some(pattern_properties),
            ..Default::default()
        };

        let yaml = serde_yaml::from_str(
            r#"
            a1b:
                name: John
        "#,
        )
        .unwrap();

        let yaml_schema = YamlSchema::typed_schema(pattern_properties_schema);
        let mut context = Context::new(&yaml_schema);
        let result = yaml_schema.validate(&mut context, &yaml);
        assert!(result.is_ok());
    }
}
