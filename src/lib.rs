use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

pub mod deser;
pub mod engine;
#[macro_use]
pub mod error;
pub mod schemas;
pub mod validation;

pub use engine::Engine;
pub use error::Error;
pub use schemas::ArraySchema;
pub use schemas::BoolOrTypedSchema;
pub use schemas::ConstSchema;
pub use schemas::EnumSchema;
pub use schemas::IntegerSchema;
pub use schemas::NumberSchema;
pub use schemas::ObjectSchema;
pub use schemas::OneOfSchema;
pub use schemas::StringSchema;
pub use validation::Context;
pub use validation::Validator;

use schemas::BooleanSchema;
use schemas::TypedSchema;

// Returns the library version, which reflects the crate version
pub fn version() -> String {
    clap::crate_version!().to_string()
}

// Alias for std::result::Result<T, yaml_schema::Error>
pub type Result<T> = std::result::Result<T, Error>;

/// A Number is either an integer or a float
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Number {
    Integer(i64),
    Float(f64),
}

impl Number {
    /// Create a new integer number
    pub fn integer(value: i64) -> Number {
        Number::Integer(value)
    }

    /// Create a new float number
    pub fn float(value: f64) -> Number {
        Number::Float(value)
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Number::Integer(v) => write!(f, "{}", v),
            Number::Float(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PropertyNamesValue {
    pub pattern: String,
}

/// A YamlSchema is either empty, a boolean, a typed schema, or an enum schema
#[derive(Debug, Default, PartialEq)]
pub enum YamlSchema {
    #[default]
    Empty,
    Boolean(bool),
    TypeNull,
    BooleanSchema(BooleanSchema),
    Const(ConstSchema),
    Enum(EnumSchema),
    OneOf(OneOfSchema),
    String(StringSchema),
    Integer(IntegerSchema),
    Number(NumberSchema),
    Object(ObjectSchema),
    Array(ArraySchema),
}

impl fmt::Display for YamlSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            YamlSchema::Empty => write!(f, "<empty schema>"),
            YamlSchema::TypeNull => write!(f, "type: null"),
            YamlSchema::Boolean(b) => write!(f, "{}", b),
            YamlSchema::BooleanSchema(b) => write!(f, "{}", b),
            YamlSchema::Const(c) => write!(f, "{}", c),
            YamlSchema::Enum(e) => write!(f, "{}", e),
            YamlSchema::Integer(i) => write!(f, "{}", i),
            YamlSchema::OneOf(one_of_schema) => {
                write!(f, "{}", one_of_schema)
            }
            YamlSchema::String(s) => write!(f, "{}", s),
            YamlSchema::Number(n) => write!(f, "{}", n),
            YamlSchema::Object(o) => write!(f, "{}", o),
            YamlSchema::Array(a) => write!(f, "{}", a),
        }
    }
}

fn deser_typed_schema(t: &crate::deser::TypedSchema) -> TypedSchema {
    match &t.r#type {
        deser::TypeValue::Single(s) => match s {
            serde_yaml::Value::String(s) => match s.as_str() {
                "string" => TypedSchema::String(StringSchema {
                    min_length: t.min_length,
                    max_length: t.max_length,
                    pattern: t.pattern.clone(),
                }),
                "number" => TypedSchema::Number(NumberSchema {
                    multiple_of: t.multiple_of,
                    exclusive_maximum: t.exclusive_maximum,
                    exclusive_minimum: t.exclusive_minimum,
                    maximum: t.maximum,
                    minimum: t.minimum,
                }),
                "array" => TypedSchema::Array(ArraySchema::from(t)),
                unknown => unimplemented!("Don't know how to deserialize type: {}", unknown),
            },
            serde_yaml::Value::Null => TypedSchema::Null,
            unsupported => panic!("Unsupported type: {:?}", unsupported),
        },
        deser::TypeValue::Array(a) => {
            unimplemented!("Can't handle multiple types yes: {}", format_vec(a))
        }
    }
}

impl From<deser::EnumSchema> for EnumSchema {
    fn from(deserialized: deser::EnumSchema) -> Self {
        EnumSchema {
            r#enum: deserialized.r#enum,
        }
    }
}

/// Formats a map of values as a string, by joining them with commas
fn format_map<V>(map: &HashMap<String, V>) -> String
where
    V: fmt::Display,
{
    let items: Vec<String> = map
        .iter()
        .map(|(k, v)| format!("\"{}\": {}", k, v))
        .collect();
    format!("{{ {} }}", items.join(", "))
}

/// Formats a vector of values as a string, by joining them with commas
fn format_vec<V>(vec: &[V]) -> String
where
    V: fmt::Display,
{
    let items: Vec<String> = vec.iter().map(|v| format!("{}", v)).collect();
    format!("[{}]", items.join(", "))
}

/// Formats a serde_yaml::Value as a string
fn format_serde_yaml_value(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::Null => "null".to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::String(s) => format!("\"{}\"", s),
        _ => format!("{:?}", value),
    }
}
