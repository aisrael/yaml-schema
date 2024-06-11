use log::debug;
use serde::{Deserialize, Serialize};

pub mod engine;
#[macro_use]
pub mod error;
pub mod literals;

pub use engine::Engine;
pub use error::YamlSchemaError;
pub use literals::{Literal, YamlString};

// Returns the library version, which reflects the crate version
pub fn version() -> String {
    clap::crate_version!().to_string()
}

pub trait Validator {
    fn validate(&self, value: &serde_yaml::Value) -> Result<(), YamlSchemaError>;
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct YamlSchema {
    pub r#type: Option<TypeValue>,
}

impl YamlSchema {
    pub fn new() -> YamlSchema {
        YamlSchema {
            ..Default::default()
        }
    }

    /// Determines whether the given `value` is accepted by the YAML schema.
    ///
    /// # Arguments
    ///
    /// * `value` - The YAML value to be checked against the schema.
    ///
    /// # Returns
    ///
    /// Returns `true` if the `value` is accepted by the schema, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_yaml::Value;
    /// use yaml_schema::YamlSchema;
    ///
    /// let schema = YamlSchema::new();
    /// let value = serde_yaml::from_str("some_yaml_string").unwrap();
    /// let accepted = schema.accepts(&value);
    /// println!("Accepted: {}", accepted);
    /// ```
    ///
    pub fn accepts(&self, value: &serde_yaml::Value) -> bool {
        debug!("Accepting value: {:?}", value);
        let engine = Engine::new(self);
        match (engine.evaluate(value)) {
            Ok(_) => true,
            Err(e) => {
                debug!("Error: {:?}", e);
                false
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TypeValue {
    String(String),
    Array(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum EnumValue {
    String(String),
    Integer(i64),
    Float(f64),
    Literal(Literal),
}

// Initialize the logger for tests
#[cfg(test)]
#[ctor::ctor]
fn init() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .format_target(false)
        .format_timestamp_secs()
        .target(env_logger::Target::Stdout)
        .init();
}

#[cfg(test)]
mod tests {

    use std::vec;

    use super::*;

    #[test]
    fn test_parse_type_only() {
        let inputs = [r#"
              type: string
            "#];
        let expecteds = [YamlSchema::Literal(Literal::String(YamlString {
            max_length: None,
            min_length: None,
            pattern: None,
        }))];
        for (expected, input) in expecteds.iter().zip(inputs.iter()) {
            let actual = serde_yaml::from_str(&format!("type: {}", input)).unwrap();
            assert_eq!(*expected, actual);
        }
    }

    fn test_parse_any_of() {
        let inputs = [r#"
            anyOf:
                - type: "string"
                  minLength: 1
            "#];
        let expecteds = [YamlSchema::AnyOf {
            any_of: vec![Literal::String(YamlString {
                max_length: None,
                min_length: Some(1),
                pattern: None,
            })],
        }];
        for (expected, input) in expecteds.iter().zip(inputs.iter()) {
            let actual = serde_yaml::from_str(input).unwrap();
            assert_eq!(*expected, actual);
        }
    }

    fn test_parse_all_of() {
        let inputs = [r#"
            allOf:
                - type: "string"
                  minLength: 1
            "#];
        let expecteds = [YamlSchema::AllOf {
            all_of: vec![Literal::String(YamlString {
                max_length: None,
                min_length: Some(1),
                pattern: None,
            })],
        }];
        for (expected, input) in expecteds.iter().zip(inputs.iter()) {
            let actual = serde_yaml::from_str(input).unwrap();
            assert_eq!(*expected, actual);
        }
    }

    fn test_parse_enum() {
        let inputs = [r#"
            enum:
                - null
            "#];
        let expecteds = [YamlSchema::Enum {
            values: vec![serde_yaml::Value::Null],
        }];
        for (expected, input) in expecteds.iter().zip(inputs.iter()) {
            let actual = serde_yaml::from_str(input).unwrap();
            assert_eq!(*expected, actual);
        }
    }

    fn test_root_string() {
        let schema: YamlSchema = serde_yaml::from_str(
            r#"
            type: string
        "#,
        )
        .unwrap();
        let expected = YamlSchema::Literal(Literal::String(YamlString {
            max_length: None,
            min_length: None,
            pattern: None,
        }));
        assert_eq!(expected, schema);
        assert!(schema
            .validate(&serde_yaml::Value::String(r#""I'm a string""#.to_string()))
            .is_ok());
    }
}
