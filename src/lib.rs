use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod deser;
pub mod engine;
#[macro_use]
pub mod error;
pub mod schemas;
pub mod validation;

pub use engine::Engine;
pub use error::Error;
pub use schemas::AnyOfSchema;
pub use schemas::ArraySchema;
pub use schemas::BoolOrTypedSchema;
pub use schemas::ConstSchema;
pub use schemas::EnumSchema;
pub use schemas::IntegerSchema;
pub use schemas::NotSchema;
pub use schemas::NumberSchema;
pub use schemas::ObjectSchema;
pub use schemas::OneOfSchema;
pub use schemas::StringSchema;
pub use validation::Context;
pub use validation::Validator;

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

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

/// YamlSchema is the core of the validation model
#[derive(Debug, Default, PartialEq)]
pub enum YamlSchema {
    #[default]
    Empty, // no value
    BooleanLiteral(bool),   // `true` or `false`
    Const(ConstSchema),     // `const`
    TypeNull,               // `type: null`
    Array(ArraySchema),     // `type: array`
    BooleanSchema,          // `type: boolean`
    Integer(IntegerSchema), // `type: integer`
    Number(NumberSchema),   // `type: number`
    Object(ObjectSchema),   // `type: object`
    String(StringSchema),   // `type: string`
    Enum(EnumSchema),       // `enum`
    AnyOf(AnyOfSchema),     // `anyOf`
    OneOf(OneOfSchema),     // `oneOf`
    Not(NotSchema),         // `not`
}

impl YamlSchema {
    pub fn boolean_literal(value: bool) -> YamlSchema {
        YamlSchema::BooleanLiteral(value)
    }
}

impl std::fmt::Display for YamlSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YamlSchema::Empty => write!(f, "<empty schema>"),
            YamlSchema::TypeNull => write!(f, "type: null"),
            YamlSchema::BooleanLiteral(b) => write!(f, "{}", b),
            YamlSchema::BooleanSchema => write!(f, "type: boolean"),
            YamlSchema::Const(c) => write!(f, "{}", c),
            YamlSchema::Enum(e) => write!(f, "{}", e),
            YamlSchema::Integer(i) => write!(f, "{}", i),
            YamlSchema::AnyOf(any_of_schema) => {
                write!(f, "{}", any_of_schema)
            }
            YamlSchema::OneOf(one_of_schema) => {
                write!(f, "{}", one_of_schema)
            }
            YamlSchema::Not(not_schema) => {
                write!(f, "{}", not_schema)
            }
            YamlSchema::String(s) => write!(f, "{}", s),
            YamlSchema::Number(n) => write!(f, "{}", n),
            YamlSchema::Object(o) => write!(f, "{}", o),
            YamlSchema::Array(a) => write!(f, "{}", a),
        }
    }
}

impl From<TypedSchema> for YamlSchema {
    fn from(schema: TypedSchema) -> Self {
        match schema {
            TypedSchema::Array(array_schema) => YamlSchema::Array(array_schema),
            TypedSchema::BooleanSchema => YamlSchema::BooleanSchema,
            TypedSchema::Null => YamlSchema::TypeNull,
            TypedSchema::Integer(integer_schema) => YamlSchema::Integer(integer_schema),
            TypedSchema::Number(number_schema) => YamlSchema::Number(number_schema),
            TypedSchema::Object(object_schema) => YamlSchema::Object(object_schema),
            TypedSchema::String(string_schema) => YamlSchema::String(string_schema),
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
    V: std::fmt::Display,
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
    V: std::fmt::Display,
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
