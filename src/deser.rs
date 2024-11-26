use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::error::YamlSchemaError;
use crate::unsupported_type;

use super::{format_map, format_vec, Number};

/// Instead of From<deser::YamlSchema>
pub trait Deser<T>: Sized {
    fn deserialize(&self) -> Result<T, YamlSchemaError>;
}

/// A YamlSchema is either empty, a boolean, a typed schema, or an enum schema
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum YamlSchema {
    #[default]
    Empty,
    Boolean(bool),
    Const(ConstSchema),
    Enum(EnumSchema),
    OneOf(OneOfSchema),
    // Need to put TypedSchema last, because not specifying `type:`
    // is interpreted as `type: null` (None)
    TypedSchema(Box<TypedSchema>),
}

/// A typed schema is a schema that has a type
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TypedSchema {
    pub r#type: TypeValue,
    // number
    pub minimum: Option<Number>,
    pub maximum: Option<Number>,
    pub exclusive_minimum: Option<Number>,
    pub exclusive_maximum: Option<Number>,
    pub multiple_of: Option<Number>,
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
    pub items: Option<ArrayItemsValue>,
    pub prefix_items: Option<Vec<YamlSchema>>,
    pub contains: Option<YamlSchema>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ConstSchema {
    pub r#const: serde_yaml::Value,
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ArrayItemsValue {
    TypedSchema(Box<TypedSchema>),
    Boolean(bool),
}

impl fmt::Display for YamlSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            YamlSchema::Empty => write!(f, "<empty schema>"),
            YamlSchema::Boolean(b) => write!(f, "{}", b),
            YamlSchema::Const(c) => write!(f, "{}", c),
            YamlSchema::Enum(e) => write!(f, "{}", e),
            YamlSchema::OneOf(one_of_schema) => {
                write!(f, "{}", one_of_schema)
            }
            YamlSchema::TypedSchema(s) => write!(f, "{}", s),
        }
    }
}

impl YamlSchema {
    pub fn new() -> YamlSchema {
        YamlSchema::Empty
    }

    pub fn const_schema<V>(value: V) -> YamlSchema
    where
        V: Into<serde_yaml::Value>,
    {
        YamlSchema::Const(ConstSchema {
            r#const: value.into(),
        })
    }

    pub fn one_of(schemas: Vec<YamlSchema>) -> YamlSchema {
        YamlSchema::OneOf(OneOfSchema { one_of: schemas })
    }

    pub fn typed_schema(schema: TypedSchema) -> YamlSchema {
        YamlSchema::TypedSchema(Box::new(schema))
    }

    pub fn is_none(&self) -> bool {
        self == &YamlSchema::Empty
    }
}

impl Deser<crate::YamlSchema> for YamlSchema {
    fn deserialize(&self) -> Result<crate::YamlSchema, YamlSchemaError> {
        match &self {
            YamlSchema::Empty => Ok(crate::YamlSchema::Empty),
            YamlSchema::Boolean(b) => Ok(crate::YamlSchema::Boolean(*b)),
            YamlSchema::Const(c) => Ok(crate::YamlSchema::Const(c.into())),
            YamlSchema::Enum(e) => Ok(crate::YamlSchema::Enum(e.into())),
            YamlSchema::OneOf(o) => Ok(crate::YamlSchema::OneOf(o.into())),
            YamlSchema::TypedSchema(t) => (*t).deserialize(),
        }
    }
}

impl TypedSchema {
    pub fn null() -> TypedSchema {
        TypedSchema {
            r#type: TypeValue::null(),
            ..Default::default()
        }
    }

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
            fields.push(format!("exclusiveMinimum: {}", ex_min));
        }
        if let Some(ex_max) = &self.exclusive_maximum {
            fields.push(format!("exclusiveMaximum: {}", ex_max));
        }
        if let Some(mult_of) = &self.multiple_of {
            fields.push(format!("multipleOf: {}", mult_of));
        }
        if let Some(props) = &self.properties {
            fields.push(format!("properties: {}", format_map(props)));
        }
        if let Some(req) = &self.required {
            fields.push(format!("required: {:?}", req));
        }
        if let Some(add_props) = &self.additional_properties {
            fields.push(format!("additionalProperties: {}", add_props));
        }
        if let Some(pattern_props) = &self.pattern_properties {
            fields.push(format!("patternProperties: {}", format_map(pattern_props)));
        }
        if let Some(min_len) = &self.min_length {
            fields.push(format!("minLength: {}", min_len));
        }
        if let Some(max_len) = &self.max_length {
            fields.push(format!("maxLength: {}", max_len));
        }
        if let Some(pattern) = &self.pattern {
            fields.push(format!("pattern: {}", pattern));
        }
        if let Some(items) = &self.items {
            fields.push(format!("items: {}", items));
        }
        if let Some(prefix_items) = &self.prefix_items {
            fields.push(format!("prefixItems: {}", format_vec(prefix_items)));
        }
        if let Some(contains) = &self.contains {
            fields.push(format!("contains: {}", contains));
        }

        write!(f, "TypedSchema {{ {} }}", fields.join(", "))
    }
}

impl Deser<crate::YamlSchema> for TypedSchema {
    fn deserialize(&self) -> Result<crate::YamlSchema, YamlSchemaError> {
        match &self.r#type {
            TypeValue::Single(s) => match s {
                serde_yaml::Value::String(s) => match s.as_str() {
                    "string" => Ok(crate::YamlSchema::String(crate::schemas::StringSchema {
                        min_length: self.min_length,
                        max_length: self.max_length,
                        pattern: self.pattern.clone(),
                    })),
                    "number" => Ok(crate::YamlSchema::Number(crate::schemas::NumberSchema {
                        multiple_of: self.multiple_of,
                        exclusive_maximum: self.exclusive_maximum,
                        exclusive_minimum: self.exclusive_minimum,
                        maximum: self.maximum,
                        minimum: self.minimum,
                    })),
                    "array" => Ok(crate::YamlSchema::Array(crate::schemas::ArraySchema::from(
                        self,
                    ))),
                    unknown => unsupported_type!("Unrecognized type '{}'!", unknown),
                },
                serde_yaml::Value::Null => Ok(crate::YamlSchema::Empty),
                unsupported => panic!("Unsupported type: {:?}", unsupported),
            },
            TypeValue::Array(a) => {
                unimplemented!("Can't handle multiple types yes: {}", format_vec(a))
            }
        }
    }
}

impl ConstSchema {
    pub fn new<V>(value: V) -> ConstSchema
    where
        V: Into<serde_yaml::Value>,
    {
        ConstSchema {
            r#const: value.into(),
        }
    }

    pub fn null() -> ConstSchema {
        ConstSchema {
            r#const: serde_yaml::Value::Null,
        }
    }

    pub fn string<V>(value: V) -> ConstSchema
    where
        V: Into<String>,
    {
        ConstSchema {
            r#const: serde_yaml::Value::String(value.into()),
        }
    }
}

impl fmt::Display for ConstSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Const {:?}", self.r#const)
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
        write!(f, "Enum {:?}", self.r#enum)
    }
}

impl fmt::Display for AdditionalProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdditionalProperties::Boolean(b) => write!(f, "additionalProperties: {}", b),
            AdditionalProperties::Type { r#type } => write!(f, "additionalProperties: {}", r#type),
        }
    }
}

impl fmt::Display for ArrayItemsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArrayItemsValue::TypedSchema(s) => write!(f, "{}", s),
            ArrayItemsValue::Boolean(b) => write!(f, "{}", b),
        }
    }
}

/// The `oneOf` schema is a schema that matches if any of the schemas in the `oneOf` array match.
/// The schemas are tried in order, and the first match is used. If no match is found, an error is added
/// to the context.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OneOfSchema {
    pub one_of: Vec<YamlSchema>,
}

impl fmt::Display for OneOfSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "oneOf:{}", format_vec(&self.one_of))
    }
}

/// A type value is either a string or an array of strings
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TypeValue {
    Single(serde_yaml::Value),
    Array(Vec<String>),
}

impl TypeValue {
    pub fn null() -> TypeValue {
        TypeValue::Single(serde_yaml::Value::Null)
    }

    pub fn from_string<V>(value: V) -> TypeValue
    where
        V: Into<String>,
    {
        TypeValue::Single(serde_yaml::Value::String(value.into()))
    }

    pub fn number() -> TypeValue {
        Self::from_string("number")
    }

    pub fn object() -> TypeValue {
        Self::from_string("object")
    }

    pub fn string() -> TypeValue {
        Self::from_string("string")
    }

    pub fn array<V>(items: Vec<V>) -> TypeValue
    where
        V: Into<String>,
    {
        let strings: Vec<String> = items.into_iter().map(|v| v.into()).collect();
        TypeValue::Array(strings)
    }

    /// Returns this TypeValue as a simple list of allowed typestrings
    ///
    /// # Examples
    ///
    /// ```
    /// use yaml_schema::deser::TypeValue;
    ///
    /// let single_type = TypeValue::from_string("string");
    /// assert_eq!(single_type.as_list_of_allowed_types(), vec!["string".to_string()]);
    ///
    /// let multiple_types = TypeValue::Array(vec!["string".to_string(), "number".to_string()]);
    /// assert_eq!(multiple_types.as_list_of_allowed_types(), vec!["string".to_string(), "number".to_string()]);
    /// ```
    pub fn as_list_of_allowed_types(&self) -> Vec<String> {
        match self {
            TypeValue::Single(s) => match s {
                serde_yaml::Value::String(s) => vec![s.clone()],
                _ => Vec::new(), // if `null`, etc., we return an empty list
            },
            TypeValue::Array(v) => v.clone(),
        }
    }
}

impl fmt::Display for TypeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeValue::Single(s) => match s {
                serde_yaml::Value::String(s) => write!(f, "\"{}\"", s),
                serde_yaml::Value::Null => write!(f, "null"),
                _ => write!(f, "{:?}", s),
            },
            TypeValue::Array(v) => write!(f, "[{}]", v.join(", ")),
        }
    }
}

impl Default for TypeValue {
    fn default() -> Self {
        TypeValue::object()
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
    fn test_parse_false_schema() {
        let schema: YamlSchema = serde_yaml::from_str("false").unwrap();
        let expected = YamlSchema::Boolean(false);
        assert_eq!(expected, schema);
    }

    #[test]
    fn test_parse_true_schema() {
        let schema: YamlSchema = serde_yaml::from_str("true").unwrap();
        let expected = YamlSchema::Boolean(true);
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
        let single_type = TypeValue::string();
        assert_eq!(
            single_type.as_list_of_allowed_types(),
            vec!["string".to_string()]
        );

        let multiple_types = TypeValue::array(vec!["string", "number"]);
        assert_eq!(
            multiple_types.as_list_of_allowed_types(),
            vec!["string".to_string(), "number".to_string()]
        );
    }

    #[test]
    fn test_null_schema() {
        let schema: YamlSchema = serde_yaml::from_str("type: null").unwrap();
        match schema {
            YamlSchema::TypedSchema(s) => {
                assert_eq!(s.r#type, TypeValue::null());
            }
            _ => panic!("Expected a TypedSchema"),
        }
    }

    #[test]
    fn test_number_schema() {
        let yaml = "
        type: number
        multipleOf: 5
        ";
        let schema: YamlSchema = serde_yaml::from_str(yaml).unwrap();
        println!("{}", schema);
    }

    #[test]
    fn test_one_of_schema() {
        let yaml = "
        oneOf:
        - type: number
          multipleOf: 5
        - type: number
          multipleOf: 3
        ";
        let schema: YamlSchema = serde_yaml::from_str(yaml).unwrap();
        println!("{}", schema);
    }
}
