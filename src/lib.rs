use std::collections::HashMap;
use std::fmt;

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

/// A YamlSchema is either empty, a boolean, a typed schema, or an enum schema
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum YamlSchema {
    #[default]
    Empty,
    Boolean(bool),
    TypedSchema(Box<TypedSchema>),
    Enum(EnumSchema),
}

/// A typed schema is a schema that has a type
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
    pub required: Option<Vec<String>>,
    pub additional_properties: Option<AdditionalProperties>,
    pub pattern_properties: Option<HashMap<String, YamlSchema>>,
    pub property_names: Option<PropertyNamesValue>,
    pub min_properties: Option<usize>,
    pub max_properties: Option<usize>,
    // string
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    // array
    pub items: Option<YamlSchema>,
}

/// A type value is either a string or an array of strings
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum AdditionalProperties {
    Boolean(bool),
    Type { r#type: TypeValue },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PropertyNamesValue {
    pub pattern: String,
}

impl YamlSchema {
    pub fn new() -> YamlSchema {
        YamlSchema::Empty
    }

    pub fn typed_schema(schema: TypedSchema) -> YamlSchema {
        YamlSchema::TypedSchema(Box::new(schema))
    }

    pub fn is_none(&self) -> bool {
        self == &YamlSchema::Empty
    }
}

impl fmt::Display for YamlSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            YamlSchema::Empty => write!(f, "<empty schema>"),
            YamlSchema::Boolean(b) => write!(f, "{}", b),
            YamlSchema::TypedSchema(s) => write!(f, "{}", s),
            YamlSchema::Enum(e) => write!(f, "{}", e),
        }
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

impl fmt::Display for TypedSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fields = Vec::new();

        fields.push(format!("type: {}", self.r#type));

        if let Some(min) = &self.minimum {
            fields.push(format!("minimum: {}", min));
        }
        if let Some(max) = &self.maximum {
            fields.push(format!("maximum: {}", max));
        }
        if let Some(ex_min) = &self.exclusive_minimum {
            fields.push(format!("exclusive_minimum: {}", ex_min));
        }
        if let Some(ex_max) = &self.exclusive_maximum {
            fields.push(format!("exclusive_maximum: {}", ex_max));
        }
        if let Some(mult_of) = &self.multiple_of {
            fields.push(format!("multiple_of: {}", mult_of));
        }
        if let Some(props) = &self.properties {
            fields.push(format!("properties: {}", format_map(props)));
        }
        if let Some(req) = &self.required {
            fields.push(format!("required: {:?}", req));
        }
        if let Some(add_props) = &self.additional_properties {
            fields.push(format!("additional_properties: {:?}", add_props));
        }
        if let Some(pattern_props) = &self.pattern_properties {
            fields.push(format!("pattern_properties: {:?}", pattern_props));
        }
        if let Some(min_len) = &self.min_length {
            fields.push(format!("min_length: {}", min_len));
        }
        if let Some(max_len) = &self.max_length {
            fields.push(format!("max_length: {}", max_len));
        }
        if let Some(pattern) = &self.pattern {
            fields.push(format!("pattern: {}", pattern));
        }

        write!(f, "TypedSchema {{ {} }}", fields.join(", "))
    }
}

// Add this function at the end of the file
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

/// A type value is either a string or an array of strings
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

    /// Returns this TypeValue as a simple list of allowed typestrings
    ///
    /// # Examples
    ///
    /// ```
    /// use yaml_schema::TypeValue;
    ///
    /// let single_type = TypeValue::String("string".to_string());
    /// assert_eq!(single_type.as_list_of_allowed_types(), vec!["string".to_string()]);
    ///
    /// let multiple_types = TypeValue::Array(vec!["string".to_string(), "number".to_string()]);
    /// assert_eq!(multiple_types.as_list_of_allowed_types(), vec!["string".to_string(), "number".to_string()]);
    /// ```
    pub fn as_list_of_allowed_types(&self) -> Vec<String> {
        match self {
            TypeValue::String(s) => vec![s.clone()],
            TypeValue::Array(v) => v.clone(),
        }
    }
}

impl fmt::Display for TypeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeValue::String(s) => write!(f, "\"{}\"", s),
            TypeValue::Array(v) => write!(f, "[{}]", v.join(", ")),
        }
    }
}

impl Default for TypeValue {
    fn default() -> Self {
        TypeValue::String("object".to_string())
    }
}

impl fmt::Display for YamlSchemaNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            YamlSchemaNumber::Integer(v) => write!(f, "{}", v),
            YamlSchemaNumber::Float(v) => write!(f, "{}", v),
        }
    }
}

impl EnumSchema {
    pub fn new<V>(values: Vec<V>) -> EnumSchema
    where
        V: Into<serde_yaml::Value>,
    {
        let values = values.into_iter().map(|v| v.into()).collect();
        EnumSchema { r#enum: values }
    }
}

impl fmt::Display for EnumSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.r#enum)
    }
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

    #[test]
    fn test_type_value_as_list_of_allowed_types() {
        let single_type = TypeValue::String("string".to_string());
        assert_eq!(
            single_type.as_list_of_allowed_types(),
            vec!["string".to_string()]
        );

        let multiple_types = TypeValue::Array(vec!["string".to_string(), "number".to_string()]);
        assert_eq!(
            multiple_types.as_list_of_allowed_types(),
            vec!["string".to_string(), "number".to_string()]
        );
    }
}
