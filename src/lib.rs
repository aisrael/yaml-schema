use serde::{Deserialize, Serialize};

pub mod engine;
#[macro_use]
pub mod error;
pub mod loader;
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

/// A RootSchema is a YamlSchema document
#[derive(Debug, Default)]
pub struct RootSchema {
    pub id: Option<String>,
    pub meta_schema: Option<String>,
    pub schema: YamlSchema,
}

impl RootSchema {
    /// Create a new RootSchema with a YamlSchema::Empty
    pub fn new(schema: YamlSchema) -> RootSchema {
        RootSchema {
            id: None,
            meta_schema: None,
            schema,
        }
    }

    /// Load a RootSchema from a file
    pub fn load_file(path: &str) -> Result<RootSchema> {
        loader::load_file(path)
    }

    pub fn load_from_str(schema: &str) -> Result<RootSchema> {
        let docs = saphyr::Yaml::load_from_str(schema)?;
        if docs.is_empty() {
            return Ok(RootSchema::new(YamlSchema::Empty)); // empty schema
        }
        loader::load_from_doc(docs.first().unwrap())
    }
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

    fn from_serde_yaml_number(value: &serde_yaml::Number) -> Self {
        if value.is_i64() {
            Number::Integer(value.as_i64().unwrap())
        } else if value.is_f64() {
            return Number::Float(value.as_f64().unwrap());
        } else {
            panic!("Expected an integer or float, but got: {:?}", value);
        }
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

#[derive(Debug, PartialEq)]
pub enum ConstValue {
    Boolean(bool),
    Null,
    Number(Number),
    String(String),
}

impl ConstValue {
    pub fn boolean(value: bool) -> ConstValue {
        ConstValue::Boolean(value)
    }
    pub fn integer(value: i64) -> ConstValue {
        ConstValue::Number(Number::integer(value))
    }
    pub fn float(value: f64) -> ConstValue {
        ConstValue::Number(Number::float(value))
    }
    pub fn null() -> ConstValue {
        ConstValue::Null
    }
    pub fn string<V: Into<String>>(value: V) -> ConstValue {
        ConstValue::String(value.into())
    }
    pub fn from_saphyr_yaml(value: &saphyr::Yaml) -> ConstValue {
        match value {
            saphyr::Yaml::Boolean(b) => ConstValue::Boolean(*b),
            saphyr::Yaml::Integer(i) => ConstValue::Number(Number::integer(*i)),
            saphyr::Yaml::Real(s) => ConstValue::Number(Number::float(s.parse::<f64>().unwrap())),
            saphyr::Yaml::String(s) => ConstValue::String(s.clone()),
            saphyr::Yaml::Null => ConstValue::Null,
            _ => panic!("Expected a constant value, but got: {:?}", value),
        }
    }
    pub fn from_serde_yaml_value(value: &serde_yaml::Value) -> ConstValue {
        match value {
            serde_yaml::Value::Bool(b) => ConstValue::Boolean(*b),
            serde_yaml::Value::Number(n) => ConstValue::Number(Number::from_serde_yaml_number(n)),
            serde_yaml::Value::String(s) => ConstValue::String(s.clone()),
            serde_yaml::Value::Null => ConstValue::Null,
            _ => panic!("Expected a constant value, but got: {:?}", value),
        }
    }
}

impl std::fmt::Display for ConstValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstValue::Boolean(b) => write!(f, "const: {}", b),
            ConstValue::Null => write!(f, "const: null"),
            ConstValue::Number(n) => write!(f, "const: {}", n),
            ConstValue::String(s) => write!(f, "const: {}", s),
        }
    }
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

/// Converts (upcast) a TypedSchema to a YamlSchema
/// Since a YamlSchema is a superset of a TypedSchema, this is a lossless conversion
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_value_from_serde_yaml_value() {
        let yaml = serde_yaml::Value::Bool(true);
        let const_value = ConstValue::from_serde_yaml_value(&yaml);
        assert_eq!(const_value, ConstValue::Boolean(true));

        let yaml = serde_yaml::Value::Number(42.into());
        let const_value = ConstValue::from_serde_yaml_value(&yaml);
        assert_eq!(const_value, ConstValue::Number(Number::integer(42)));

        let yaml = serde_yaml::Value::String("Drive".to_string());
        let const_value = ConstValue::from_serde_yaml_value(&yaml);
        assert_eq!(const_value, ConstValue::String("Drive".to_string()));
    }
}
