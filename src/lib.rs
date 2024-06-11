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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum YamlSchema {
    Boolean(bool),
    Schema(YamlSchemaValue),
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct YamlSchemaValue {
    pub r#type: Option<TypeValue>,
}

impl YamlSchema {
    pub fn new() -> YamlSchema {
        YamlSchema::Schema(YamlSchemaValue {
            ..Default::default()
        })
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

impl Validator for YamlSchema {
    fn validate(&self, value: &serde_yaml::Value) -> Result<(), YamlSchemaError> {
        debug!("Validating value: {:?}", value);
        match self {
            YamlSchema::Boolean(boolean) => {
                if *boolean {
                    Ok(())
                } else {
                    generic_error!("Schema is `false`!")
                }
            }
            YamlSchema::Schema(schema_value) => {
                debug!("Schema value: {:?}", schema_value);
                Ok(())
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
    fn test_parse_empty_schema() {
        let schema: YamlSchema = serde_yaml::from_str("{}").unwrap();
        let expected = YamlSchema::Schema(YamlSchemaValue {
            r#type: None,
        });
        assert_eq!(expected, schema);
    }

    #[test]
    fn test_parse_true_schema() {
        let schema: YamlSchema = serde_yaml::from_str("true").unwrap();
        let expected = YamlSchema::Boolean(true);
        assert_eq!(expected, schema);
    }


    #[test]
    fn test_parse_false_schema() {
        let schema: YamlSchema = serde_yaml::from_str("false").unwrap();
        let expected = YamlSchema::Boolean(false);
        assert_eq!(expected, schema);
    }
}
