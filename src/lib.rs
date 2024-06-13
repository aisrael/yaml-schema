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


impl YamlSchema {
    pub fn new() -> YamlSchema {
        YamlSchema::Schema(YamlSchemaValue {
            ..Default::default()
        })
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
                match schema_value.r#type.as_deref() {
                    Some("string") => {
                        if let serde_yaml::Value::String(_) = value {
                            Ok(())
                        } else {
                            generic_error!("Value is not a string!")
                        }
                    }
                    _ => Ok(()),

                }
            }
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct YamlSchemaValue {
    pub r#type: Option<String>,
}

impl YamlSchemaValue {
    pub fn for_type(s: &str) -> YamlSchemaValue {
        YamlSchemaValue {
            r#type: Some(s.to_string()),
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

    use super::*;

    #[test]
    fn test_parse_empty_schema() {
        let schema: Option<YamlSchema> = serde_yaml::from_str("").unwrap();
        assert!(schema.is_none());
        let schema: YamlSchema = serde_yaml::from_str("{}").unwrap();
        let expected = YamlSchema::Schema(YamlSchemaValue { r#type: None });
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

    #[test]
    fn test_parse_type_string_schema() {
        let schema: YamlSchema = serde_yaml::from_str("type: string").unwrap();
        let expected = YamlSchema::Schema(YamlSchemaValue::for_type("string"));
        assert_eq!(expected, schema);
    }
}
