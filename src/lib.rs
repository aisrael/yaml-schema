use std::rc::Rc;

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
    pub schema: Rc<YamlSchema>,
}

impl RootSchema {
    /// Create a new RootSchema with a YamlSchema::Empty
    pub fn new(schema: YamlSchema) -> RootSchema {
        RootSchema {
            id: None,
            meta_schema: None,
            schema: Rc::new(schema),
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

    pub fn validate(&self, context: &Context, value: &saphyr::MarkedYaml) -> Result<()> {
        self.schema.validate(context, value)?;
        Ok(())
    }
}

/// A Number is either an integer or a float
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Number {
    Integer(i64),
    Float(f64),
}

impl Number {
    /// Create a new integer Number
    pub fn integer(value: i64) -> Number {
        Number::Integer(value)
    }

    /// Create a new float Number
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
}

impl TryFrom<&saphyr::YamlData<saphyr::MarkedYaml>> for ConstValue {
    type Error = crate::Error;

    fn try_from(value: &saphyr::YamlData<saphyr::MarkedYaml>) -> Result<Self> {
        match value {
            saphyr::YamlData::String(s) => Ok(ConstValue::String(s.clone())),
            saphyr::YamlData::Integer(i) => Ok(ConstValue::Number(Number::integer(*i))),
            saphyr::YamlData::Real(f) => {
                let f = f.parse::<f64>()?;
                Ok(ConstValue::Number(Number::float(f)))
            }
            saphyr::YamlData::Boolean(b) => Ok(ConstValue::Boolean(*b)),
            saphyr::YamlData::Null => Ok(ConstValue::Null),
            v => Err(unsupported_type!(
                "Expected a constant value, but got: {:?}",
                v
            )),
        }
    }
}

impl TryFrom<saphyr::Yaml> for ConstValue {
    type Error = crate::Error;

    fn try_from(value: saphyr::Yaml) -> Result<Self> {
        match value {
            saphyr::Yaml::Boolean(b) => Ok(ConstValue::Boolean(b)),
            saphyr::Yaml::Integer(i) => Ok(ConstValue::Number(Number::integer(i))),
            saphyr::Yaml::Real(s) => {
                let f = s.parse::<f64>()?;
                Ok(ConstValue::Number(Number::float(f)))
            }
            saphyr::Yaml::String(s) => Ok(ConstValue::String(s.clone())),
            saphyr::Yaml::Null => Ok(ConstValue::Null),
            v => Err(unsupported_type!(
                "Expected a constant value, but got: {:?}",
                v
            )),
        }
    }
}

impl std::fmt::Display for ConstValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstValue::Boolean(b) => write!(f, "{} (bool)", b),
            ConstValue::Null => write!(f, "null"),
            ConstValue::Number(n) => write!(f, "{} (number)", n),
            ConstValue::String(s) => write!(f, "\"{}\"", s),
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

/// Formats a saphyr::YamlData as a string
fn format_yaml_data(data: &saphyr::YamlData<saphyr::MarkedYaml>) -> String {
    match data {
        saphyr::YamlData::Null => "null".to_string(),
        saphyr::YamlData::Boolean(b) => b.to_string(),
        saphyr::YamlData::Integer(i) => i.to_string(),
        saphyr::YamlData::Real(s) => s.clone(),
        saphyr::YamlData::String(s) => format!("\"{}\"", s),
        _ => format!("{:?}", data),
    }
}

fn format_marker(marker: &saphyr::Marker) -> String {
    format!("[{}, {}]", marker.line(), marker.col())
}

/// Use the ctor crate to initialize the logger for tests
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
    fn test_const_equality() {
        let i1 = ConstValue::integer(42);
        let i2 = ConstValue::integer(42);
        assert_eq!(i1, i2);

        let s1 = ConstValue::string("NW");
        let s2 = ConstValue::string("NW");
        assert_eq!(s1, s2);
    }
}
