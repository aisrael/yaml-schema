use log::{debug, error};

use super::validation::objects::try_validate_value_against_properties;
use super::validation::strings::validate_string;
use crate::error::YamlSchemaError;
use crate::validation::objects::try_validate_value_against_additional_properties;
pub use crate::validation::Context;
pub use crate::validation::ValidationError;
use crate::{
    fail_fast, format_vec, generic_error, not_yet_implemented, ArrayItemsValue, ConstSchema,
    EnumSchema, OneOfSchema, TypeValue, TypedSchema, YamlSchema, YamlSchemaNumber,
};

pub struct Engine<'a> {
    pub schema: &'a YamlSchema,
}

impl<'a> Engine<'a> {
    pub fn new(schema: &'a YamlSchema) -> Engine<'a> {
        Engine { schema }
    }

    pub fn evaluate(
        &self,
        yaml: &serde_yaml::Value,
        fail_fast: bool,
    ) -> Result<Context, YamlSchemaError> {
        debug!("Engine is running");
        let context = Context::new(fail_fast);
        let result = self.schema.validate(&context, yaml);
        debug!("Engine: result: {:?}", result);
        debug!("Engine: context.errors: {}", context.errors.borrow().len());
        match result {
            Ok(()) | Err(YamlSchemaError::FailFast) => Ok(context),
            Err(e) => Err(e),
        }
    }
}

pub trait Validator {
    fn validate(&self, context: &Context, value: &serde_yaml::Value)
        -> Result<(), YamlSchemaError>;
}

impl Validator for YamlSchema {
    fn validate(
        &self,
        context: &Context,
        value: &serde_yaml::Value,
    ) -> Result<(), YamlSchemaError> {
        debug!("YamlSchema: self: {}", self);
        debug!("YamlSchema: Validating value: {:?}", value);
        match self {
            YamlSchema::Empty => Ok(()),
            YamlSchema::Boolean(boolean) => {
                if !*boolean {
                    context.add_error("Schema is `false`!".to_string());
                }
                Ok(())
            }
            YamlSchema::Const(const_schema) => const_schema.validate(context, value),
            YamlSchema::TypedSchema(typed_schema) => {
                debug!("YamlSchema: Schema value: {}", typed_schema);
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
        context: &Context,
        value: &serde_yaml::Value,
    ) -> Result<(), YamlSchemaError> {
        debug!("TypedSchema: Validating value: {:?}", value);

        match self.r#type {
            TypeValue::Single(ref v) => match v {
                serde_yaml::Value::String(ref s) => match s.as_str() {
                    "array" => self.validate_array(context, value),
                    "boolean" => self.validate_boolean(context, value),
                    "integer" => {
                        self.validate_integer(context, value);
                        Ok(())
                    }
                    "number" => {
                        self.validate_number(context, value);
                        Ok(())
                    }
                    "object" => self.validate_object(context, value),
                    "string" => self.validate_string(context, value),
                    _ => Err(generic_error!("Unknown type '{}'!", s)),
                },
                serde_yaml::Value::Null => {
                    if !value.is_null() {
                        context.add_error(format!("Expected a value, but got: {:?}", value));
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
    fn validate_boolean(
        &self,
        context: &Context,
        value: &serde_yaml::Value,
    ) -> Result<(), YamlSchemaError> {
        if !value.is_bool() {
            context.add_error(format!("Expected a boolean, but got: {:?}", value));
            fail_fast!(context);
        }
        Ok(())
    }

    fn validate_integer(&self, context: &Context, value: &serde_yaml::Value) {
        match value.as_i64() {
            Some(i) => self.validate_number_i64(context, i),
            None => {
                if value.is_f64() {
                    let f = value.as_f64().unwrap();
                    if f.fract() == 0.0 {
                        return self.validate_number_i64(context, f as i64);
                    } else {
                        context.add_error(format!("Expected an integer, but got: {:?}", value));
                    }
                }
                context.add_error(format!("Expected an integer, but got: {:?}", value));
            }
        }
    }

    fn validate_number(&self, context: &Context, value: &serde_yaml::Value) {
        if value.is_i64() {
            match value.as_i64() {
                Some(i) => self.validate_number_i64(context, i),
                None => {
                    context.add_error(format!("Expected an integer, but got: {:?}", value));
                }
            }
        } else if value.is_f64() {
            match value.as_f64() {
                Some(f) => self.validate_number_f64(context, f),
                None => {
                    context.add_error(format!("Expected a float, but got: {:?}", value));
                }
            }
        } else {
            context.add_error(format!("Expected a number, but got: {:?}", value));
        }
    }

    fn validate_number_i64(&self, context: &Context, i: i64) {
        if let Some(minimum) = &self.minimum {
            match minimum {
                YamlSchemaNumber::Integer(min) => {
                    if i < *min {
                        context.add_error("Number is too small!".to_string());
                    }
                }
                YamlSchemaNumber::Float(min) => {
                    if (i as f64) < *min {
                        context.add_error("Number is too small!".to_string());
                    }
                }
            }
        }
        if let Some(maximum) = &self.maximum {
            match maximum {
                YamlSchemaNumber::Integer(max) => {
                    if i > *max {
                        context.add_error("Number is too big!".to_string());
                    }
                }
                YamlSchemaNumber::Float(max) => {
                    if (i as f64) > *max {
                        context.add_error("Number is too big!".to_string());
                    }
                }
            }
        }
        if let Some(multiple_of) = &self.multiple_of {
            match multiple_of {
                YamlSchemaNumber::Integer(multiple) => {
                    if i % *multiple != 0 {
                        context.add_error(format!("Number is not a multiple of {}!", multiple));
                    }
                }
                YamlSchemaNumber::Float(multiple) => {
                    if (i as f64) % *multiple != 0.0 {
                        context.add_error(format!("Number is not a multiple of {}!", multiple));
                    }
                }
            }
        }
    }
    fn validate_number_f64(&self, context: &Context, f: f64) {
        if let Some(minimum) = &self.minimum {
            match minimum {
                YamlSchemaNumber::Integer(min) => {
                    if f < *min as f64 {
                        context.add_error("Number is too small!".to_string());
                    }
                }
                YamlSchemaNumber::Float(min) => {
                    if f < *min {
                        context.add_error("Number is too small!".to_string());
                    }
                }
            }
        }
        if let Some(maximum) = &self.maximum {
            match maximum {
                YamlSchemaNumber::Integer(max) => {
                    if f > *max as f64 {
                        context.add_error("Number is too big!".to_string());
                    }
                }
                YamlSchemaNumber::Float(max) => {
                    if f > *max {
                        context.add_error("Number is too big!".to_string());
                    }
                }
            }
        }
    }

    /// Validate the string according to the schema rules
    fn validate_string(
        &self,
        context: &Context,
        value: &serde_yaml::Value,
    ) -> Result<(), YamlSchemaError> {
        match validate_string(
            self.min_length,
            self.max_length,
            self.pattern.as_ref(),
            value,
        ) {
            Ok(errors) => {
                if !errors.is_empty() {
                    for error in errors {
                        debug!("validate_string: error: {}", error);
                        context.add_error(error);
                    }
                    fail_fast!(context);
                }
                Ok(())
            }
            Err(e) => {
                let s = e.to_string();
                error!("{}", s);
                Err(YamlSchemaError::GenericError(s))
            }
        }
    }

    /// Validate the object according to the schema rules
    fn validate_object(
        &self,
        context: &Context,
        value: &serde_yaml::Value,
    ) -> Result<(), YamlSchemaError> {
        debug!("Validating object: {:?}", value);
        match value.as_mapping() {
            Some(mapping) => self.validate_object_mapping(context, mapping),
            None => {
                context.add_error("Expected an object, but got: None");
                Ok(())
            }
        }
    }

    fn validate_object_mapping(
        &self,
        context: &Context,
        mapping: &serde_yaml::Mapping,
    ) -> Result<(), YamlSchemaError> {
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
                if !try_validate_value_against_additional_properties(
                    context,
                    &key,
                    value,
                    additional_properties,
                )? {
                    return Ok(());
                }
            }
            // Then we check if pattern_properties matches
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
                        schema.validate(context, value)?;
                    }
                }
            }
            // Finally, we check if it matches property_names
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
        context: &Context,
        value: &serde_yaml::Value,
    ) -> Result<(), YamlSchemaError> {
        if !value.is_sequence() {
            context.add_error(format!("Expected an array, but got: {:?}", value));
            fail_fast!(context);
            return Ok(());
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
        context: &Context,
        value: &serde_yaml::Value,
    ) -> Result<(), YamlSchemaError> {
        debug!(
            "Validating value: {:?} against const: {:?}",
            value, self.r#const
        );
        let expected_value = &self.r#const;
        if expected_value != value {
            let error = format!(
                "Const validation failed, expected: {:?}, got: {:?}",
                expected_value, value
            );
            context.add_error(error);
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
        context: &Context,
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
            let error = format!("Value {} is not in the enum: [{}]", value_str, enum_values);
            context.add_error(error);
        }
        Ok(())
    }
}

impl Validator for OneOfSchema {
    fn validate(
        &self,
        context: &Context,
        value: &serde_yaml::Value,
    ) -> Result<(), YamlSchemaError> {
        let schemas: &Vec<YamlSchema> = &self.one_of;
        let mut one_of_is_valid = false;
        for schema in schemas {
            debug!(
                "OneOf: Validating value: {:?} against schema: {}",
                value, schema
            );
            let sub_context = Context::new(context.fail_fast);
            let sub_result = schema.validate(&sub_context, value);
            match sub_result {
                Ok(()) | Err(YamlSchemaError::FailFast) => {
                    debug!(
                        "OneOf: sub_context.errors: {}",
                        sub_context.errors.borrow().len()
                    );
                    if sub_context.has_errors() {
                        continue;
                    }

                    if one_of_is_valid {
                        error!("OneOf: Value matched multiple schemas in `oneOf`!");
                        context.add_error("Value matched multiple schemas in `oneOf`!");
                        fail_fast!(context);
                    } else {
                        one_of_is_valid = true;
                    }
                }
                Err(e) => return Err(e),
            }
        }
        debug!("OneOf: one_of_is_valid: {}", one_of_is_valid);
        if !one_of_is_valid {
            error!("OneOf: None of the schemas in `oneOf` matched!");
            context.add_error("None of the schemas in `oneOf` matched!");
            fail_fast!(context);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use crate::AdditionalProperties;

    use super::*;

    #[test]
    fn test_const() {
        let const_schema = ConstSchema::new("United States of America");
        let context = Context::new(true);
        assert!(const_schema
            .validate(
                &context,
                &serde_yaml::Value::String("United States of America".to_string())
            )
            .is_ok());
        assert!(!context.has_errors());
        let _ = const_schema.validate(&context, &serde_yaml::Value::String("Canada".to_string()));
        assert!(context.has_errors());
        let error = context.errors.borrow_mut().pop().unwrap();
        assert_eq!(error.error, "Const validation failed, expected: String(\"United States of America\"), got: String(\"Canada\")");

        let schema = TypedSchema::object(
            vec![(
                "country".to_string(),
                YamlSchema::const_schema("United States of America"),
            )]
            .into_iter()
            .collect(),
        );
        let yaml = serde_yaml::from_str(
            r#"
            country: United States of America
        "#,
        )
        .unwrap();
        let result = schema.validate(&context, &yaml);
        assert!(result.is_ok());
        assert!(!context.has_errors());

        let parsed_yaml_schema: YamlSchema = serde_yaml::from_str(
            r#"
            const: United States of America
            "#,
        )
        .unwrap();
        let result = parsed_yaml_schema.validate(
            &context,
            &serde_yaml::Value::String("United States of America".to_string()),
        );
        assert!(result.is_ok());
        assert!(!context.has_errors());
    }

    #[test]
    fn test_type_object_should_validate_properties() {
        // Equivalent to:
        //
        //   type: object
        //   properties:
        //     foo:
        //       type: string
        //     bar:
        //       type: number
        let schema = TypedSchema::object(
            vec![
                (
                    "foo".to_string(),
                    YamlSchema::TypedSchema(Box::new(TypedSchema::string())),
                ),
                (
                    "bar".to_string(),
                    YamlSchema::TypedSchema(Box::new(TypedSchema::number())),
                ),
            ]
            .into_iter()
            .collect(),
        );
        let yaml = serde_yaml::from_str(
            r#"
            foo: 42
            bar: "I'm a string"
            "#,
        )
        .unwrap();
        let context = Context::new(false);
        let result = schema.validate(&context, &yaml);
        assert!(result.is_ok());
        assert!(context.has_errors());
        let errors = context.errors.borrow();
        debug!("Got {} errors", errors.len());
        assert_eq!(2, errors.len());
        if !errors.is_empty() {
            for error in errors.iter() {
                debug!("test_type_object_should_validate_properties: {}", error);
            }
        }
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
        assert!(engine.evaluate(&yaml, true).is_ok());
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
        println!("Testing valid yaml");
        let result = engine.evaluate(&yaml, true);
        assert!(result.is_ok());
        let context = result.unwrap();
        assert!(!context.has_errors(), "Expected no errors, but got some");

        let invalid_yaml = serde_yaml::from_str(
            r#"
            age: 42
        "#,
        )
        .unwrap();
        println!("Testing additional number property should fail");
        let invalid_result = engine.evaluate(&invalid_yaml, true);
        assert!(
            invalid_result.is_ok(),
            "Expected validation to succeed, but got an error"
        );
        let context = invalid_result.unwrap();
        assert!(context.has_errors(), "Expected errors, but got none");
        let error = context.errors.borrow_mut().pop().unwrap();
        println!("Error: {}", error);
        assert_eq!(error.path, ""); // in the root
        assert_eq!(
            error.error,
            "Additional property 'age' is not allowed. No allowed types matched!"
        );
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
        let result = engine.evaluate(&yaml, true);
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
        let context = Context::new(true);
        let result = one_of_schema.validate(&context, &yaml);
        assert!(result.is_ok());
        for error in context.errors.borrow().iter() {
            println!("Error: {:?}", error.error);
        }
        assert!(!context.has_errors(), "Expected no errors, but got some");
    }

    #[test]
    fn test_child_of_one_of() {
        let object_schema = TypedSchema {
            r#type: TypeValue::object(),
            properties: Some(HashMap::from([(
                "name".to_string(),
                YamlSchema::TypedSchema(Box::new(TypedSchema::string())),
            )])),
            required: Some(vec!["name".to_string()]),
            ..Default::default()
        };
        let context = Context::new(true);
        let yaml = serde_yaml::from_str(
            r#"
            name: "John Doe"
        "#,
        )
        .unwrap();
        let result = object_schema.validate(&context, &yaml);
        assert!(result.is_ok());
        assert!(!context.has_errors(), "Expected no errors, but got some");

        let yaml = serde_yaml::from_str(r#"null"#).unwrap();
        let result = object_schema.validate(&context, &yaml);
        assert!(result.is_ok());
        assert!(context.has_errors(), "Expected errors, but got none");
    }

    #[test]
    fn test_one_of_null_or_object() {
        let object_schema = TypedSchema {
            r#type: TypeValue::object(),
            properties: Some(HashMap::from([(
                "name".to_string(),
                YamlSchema::TypedSchema(Box::new(TypedSchema::string())),
            )])),
            required: Some(vec!["name".to_string()]),
            ..Default::default()
        };
        let schemas: Vec<YamlSchema> = vec![
            YamlSchema::TypedSchema(Box::new(TypedSchema::null())),
            YamlSchema::TypedSchema(Box::new(object_schema)),
        ];
        let one_of_schema = YamlSchema::one_of(schemas);
        let properties = HashMap::from([("child".to_string(), one_of_schema)]);
        let typed_schema = TypedSchema {
            r#type: TypeValue::object(),
            properties: Some(properties),
            additional_properties: Some(AdditionalProperties::Boolean(false)),
            ..Default::default()
        };
        let context = Context::new(true);
        let yaml = serde_yaml::from_str(
            r#"
            child: null
        "#,
        )
        .unwrap();
        let result = typed_schema.validate(&context, &yaml);
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

        let context = Context::new(true);
        let result = pattern_properties_schema.validate(&context, &yaml);
        assert!(result.is_ok());
    }
}
