pub mod deser;
pub mod engine;
#[macro_use]
pub mod error;
pub mod schemas;
pub mod validation;

pub use engine::Engine;
pub use error::YamlSchemaError;
pub use schemas::{
    ArraySchema, ConstSchema, EnumSchema, NumberSchema, ObjectSchema, OneOfSchema, StringSchema,
};
use schemas::{BooleanSchema, TypedSchema};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};
pub use validation::{Context, Validator};

// Returns the library version, which reflects the crate version
pub fn version() -> String {
    clap::crate_version!().to_string()
}

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

/// A YamlSchema is either empty, a boolean, a typed schema, or an enum schema
#[derive(Debug, Default, PartialEq)]
pub enum YamlSchema {
    #[default]
    Empty,
    Boolean(bool),
    BooleanSchema(BooleanSchema),
    Const(ConstSchema),
    Enum(EnumSchema),
    OneOf(OneOfSchema),
    String(StringSchema),
    Number(NumberSchema),
    Object(ObjectSchema),
    Array(ArraySchema),
}

impl fmt::Display for YamlSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            YamlSchema::Empty => write!(f, "<empty schema>"),
            YamlSchema::Boolean(b) => write!(f, "{}", b),
            YamlSchema::BooleanSchema(b) => write!(f, "{}", b),
            YamlSchema::Const(c) => write!(f, "{}", c),
            YamlSchema::Enum(e) => write!(f, "{}", e),
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

impl From<&crate::deser::YamlSchema> for YamlSchema {
    fn from(deserialized: &crate::deser::YamlSchema) -> Self {
        match deserialized {
            deser::YamlSchema::Empty => YamlSchema::Empty,
            deser::YamlSchema::Boolean(b) => YamlSchema::Boolean(*b),
            deser::YamlSchema::Const(c) => YamlSchema::Const(c.into()),
            deser::YamlSchema::Enum(e) => YamlSchema::Enum(e.into()),
            deser::YamlSchema::OneOf(o) => YamlSchema::OneOf(o.into()),
            deser::YamlSchema::TypedSchema(t) => match deser_typed_schema(&t) {
                TypedSchema::Array(a) => YamlSchema::Array(a),
                TypedSchema::Boolean => YamlSchema::BooleanSchema(BooleanSchema),
                TypedSchema::Number(n) => YamlSchema::Number(n),
                TypedSchema::Object(o) => YamlSchema::Object(o),
                TypedSchema::String(s) => YamlSchema::String(s),
            },
        }
    }
}

impl From<&crate::deser::TypedSchema> for TypedSchema {
    fn from(value: &crate::deser::TypedSchema) -> Self {
        match &value.r#type {
            crate::deser::TypeValue::Single(s) => deser_typed_schema(value),
            crate::deser::TypeValue::Array(a) => unimplemented!(),
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
            not_yet => unimplemented!("Don't know how to deserialize type: {:?}", not_yet),
        },
        _ => unimplemented!(),
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
