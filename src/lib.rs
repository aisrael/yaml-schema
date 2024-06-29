use std::collections::HashMap;

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

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum YamlSchema {
    #[default]
    Empty,
    Boolean(bool),
    TypedSchema(Box<TypedSchema>),
    Enum(EnumSchema),
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TypedSchema {
    pub r#type: TypeValue,
    // number
    pub minimum: Option<YamlSchemaNumber>,
    pub maximum: Option<YamlSchemaNumber>,
    pub exclusive_minimum: Option<YamlSchemaNumber>,
    pub exclusive_maximum: Option<YamlSchemaNumber>,
    pub multiple_of: Option<YamlSchemaNumber>,
    // object
    pub properties: Option<HashMap<String, YamlSchema>>,
    // string
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TypeValue {
    String(String),
    Array(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum YamlSchemaNumber {
    Integer(i64),
    Float(f64),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct EnumSchema {
    pub r#enum: Vec<serde_yaml::Value>,
}

impl YamlSchema {
    pub fn new() -> YamlSchema {
        YamlSchema::Empty
    }

    pub fn is_none(&self) -> bool {
        self == &YamlSchema::Empty
    }
}

impl TypedSchema {
    pub fn string() -> TypedSchema {
        TypedSchema {
            r#type: TypeValue::string(),
            ..Default::default()
        }
    }

    pub fn number() -> TypedSchema {
        TypedSchema {
            r#type: TypeValue::number(),
            ..Default::default()
        }
    }

    pub fn object(properties: HashMap<String, YamlSchema>) -> TypedSchema {
        TypedSchema {
            r#type: TypeValue::object(),
            properties: Some(properties),
            ..Default::default()
        }
    }
}

impl TypeValue {
    pub fn number() -> TypeValue {
        TypeValue::String("number".to_string())
    }

    pub fn object() -> TypeValue {
        TypeValue::String("object".to_string())
    }

    pub fn string() -> TypeValue {
        TypeValue::String("string".to_string())
    }
}

impl Default for TypeValue {
    fn default() -> Self {
        TypeValue::String("object".to_string())
    }
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
        let schema: YamlSchema = serde_yaml::from_str("").unwrap();
        assert!(schema.is_none());
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
        let expected = YamlSchema::TypedSchema(Box::new(TypedSchema::string()));
        assert_eq!(expected, schema);
    }
}
