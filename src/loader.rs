/// Loader defines the interface for loading a schema from a file or a string
use log::debug;
use std::collections::HashMap;
use std::fs;

use crate::schemas::TypedSchema;
use crate::ArraySchema;
use crate::BoolOrTypedSchema;
use crate::ConstSchema;
use crate::ConstValue;
use crate::EnumSchema;
use crate::Error;
use crate::IntegerSchema;
use crate::Number;
use crate::NumberSchema;
use crate::ObjectSchema;
use crate::Result;
use crate::RootSchema;
use crate::StringSchema;
use crate::YamlSchema;

pub fn load_file<S: Into<String>>(path: S) -> Result<RootSchema> {
    let path_s = path.into();
    let fs_metadata = fs::metadata(&path_s)?;
    if !fs_metadata.is_file() {
        return Err(Error::FileNotFound(path_s.clone()));
    }
    let s = fs::read_to_string(&path_s)?;
    let docs = saphyr::Yaml::load_from_str(&s)?;
    if docs.is_empty() {
        return Ok(RootSchema::new(YamlSchema::Empty)); // empty schema
    }
    load_from_doc(docs.first().unwrap())
}

pub fn load_from_doc(doc: &saphyr::Yaml) -> Result<RootSchema> {
    let mut loader = RootLoader::new();
    match doc {
        saphyr::Yaml::Boolean(r#bool) => {
            loader.set_schema(YamlSchema::BooleanLiteral(*r#bool));
        }
        saphyr::Yaml::Hash(hash) => {
            loader.load_root_schema(hash)?;
        }
        saphyr::Yaml::Null => {
            loader.set_schema(YamlSchema::TypeNull);
        }
        saphyr::Yaml::String(s) => match s.as_str() {
            "true" => {
                loader.set_schema(YamlSchema::BooleanLiteral(true));
            }
            "false" => {
                loader.set_schema(YamlSchema::BooleanLiteral(false));
            }
            s => {
                println!("s: {:#?}", s);
                unimplemented!()
            }
        },
        _ => {
            println!("doc: {:#?}", doc);
            unimplemented!()
        }
    }
    Ok(loader.into()) // See From<Loader> for RootSchema below
}

#[derive(Debug, Default)]
struct RootLoader {
    pub id: Option<String>,
    pub meta_schema: Option<String>,
    pub schema: Option<YamlSchema>,
}

impl RootLoader {
    fn new() -> Self {
        RootLoader::default()
    }

    /// Set the loader schema
    /// Just a convenience function to avoid having to write
    /// `self.schema = Some(schema);`
    fn set_schema(&mut self, schema: YamlSchema) {
        self.schema = Some(schema);
    }

    fn load_root_schema(&mut self, hash: &saphyr::Hash) -> Result<()> {
        let id_key = saphyr::Yaml::String(String::from("$id"));
        let schema_key = saphyr::Yaml::String(String::from("$schema"));
        if let Some(id) = hash.get(&id_key) {
            self.id = Some(yaml_to_string(id, "$id value must be a string")?);
        }
        if let Some(schema) = hash.get(&schema_key) {
            self.meta_schema = Some(yaml_to_string(schema, "$schema value must be a string")?);
        }
        self.schema = load_sub_schema(hash)?;
        Ok(())
    }
}

fn load_sub_schema(hash: &saphyr::Hash) -> Result<Option<YamlSchema>> {
    let type_key = sys("type");
    if hash.contains_key(&type_key) {
        let value = hash.get(&type_key).unwrap();
        match value {
            saphyr::Yaml::String(s) => {
                debug!("Loading type: {}", s);
                match s.as_str() {
                    "array" => {
                        let array_schema = ArraySchema::construct(hash)?;
                        return Ok(Some(YamlSchema::Array(array_schema)));
                    }
                    "boolean" => return Ok(Some(YamlSchema::BooleanSchema)),
                    "enum" => {
                        let enum_schema = EnumSchema::construct(hash)?;
                        return Ok(Some(YamlSchema::Enum(enum_schema)));
                    }
                    "integer" => {
                        let integer_schema = IntegerSchema::construct(hash)?;
                        return Ok(Some(YamlSchema::Integer(integer_schema)));
                    }
                    "number" => {
                        let number_schema = NumberSchema::construct(hash)?;
                        return Ok(Some(YamlSchema::Number(number_schema)));
                    }
                    "object" => {
                        let object_schema = ObjectSchema::construct(hash)?;
                        return Ok(Some(YamlSchema::Object(object_schema)));
                    }
                    "string" => {
                        let string_schema = StringSchema::construct(hash)?;
                        return Ok(Some(YamlSchema::String(string_schema)));
                    }
                    _ => return Err(unsupported_type!(s.to_string())),
                }
            }
            saphyr::Yaml::Null => {
                return Ok(Some(YamlSchema::TypeNull));
            }
            _ => unimplemented!(),
        }
    } else {
        let enum_key = sys("enum");
        if hash.contains_key(&enum_key) {
            let value = hash.get(&enum_key).unwrap();
            match value {
                saphyr::Yaml::Array(array) => {
                    let enum_values = array
                        .iter()
                        .map(|v| match v {
                            saphyr::Yaml::String(s) => Ok(ConstValue::string(s)),
                            saphyr::Yaml::Integer(i) => Ok(ConstValue::integer(*i)),
                            saphyr::Yaml::Real(s) => {
                                let f = s.parse::<f64>()?;
                                Ok(ConstValue::float(f))
                            }
                            saphyr::Yaml::Null => Ok(ConstValue::null()),
                            _ => unimplemented!(),
                        })
                        .collect::<Result<Vec<ConstValue>>>()?;
                    return Ok(Some(YamlSchema::Enum(EnumSchema {
                        r#enum: enum_values,
                    })));
                }
                _ => unimplemented!(),
            }
        } else {
            let const_key = sys("const");
            if hash.contains_key(&const_key) {
                let value = hash.get(&const_key).unwrap();
                match value {
                    saphyr::Yaml::String(s) => {
                        return Ok(Some(YamlSchema::Const(ConstSchema {
                            r#const: ConstValue::string(s),
                        })));
                    }
                    saphyr::Yaml::Integer(i) => {
                        return Ok(Some(YamlSchema::Const(ConstSchema {
                            r#const: ConstValue::integer(*i),
                        })));
                    }
                    saphyr::Yaml::Real(s) => {
                        let f = s.parse::<f64>()?;
                        return Ok(Some(YamlSchema::Const(ConstSchema {
                            r#const: ConstValue::float(f),
                        })));
                    }
                    _ => unimplemented!(),
                }
            }
        }
    }
    Ok(None)
}

fn load_typed_schema(hash: &saphyr::Hash) -> Result<Option<TypedSchema>> {
    let type_key = saphyr::Yaml::String(String::from("type"));
    if hash.contains_key(&type_key) {
        let value = hash.get(&type_key).unwrap();
        match value {
            saphyr::Yaml::String(s) => match s.as_str() {
                "array" => {
                    let array_schema = ArraySchema::construct(hash)?;
                    return Ok(Some(TypedSchema::Array(array_schema)));
                }
                "boolean" => return Ok(Some(TypedSchema::BooleanSchema)),
                "integer" => {
                    let integer_schema = IntegerSchema::construct(hash)?;
                    return Ok(Some(TypedSchema::Integer(integer_schema)));
                }
                "number" => {
                    let number_schema = NumberSchema::construct(hash)?;
                    return Ok(Some(TypedSchema::Number(number_schema)));
                }
                "object" => {
                    let object_schema = ObjectSchema::construct(hash)?;
                    return Ok(Some(TypedSchema::Object(object_schema)));
                }
                "string" => {
                    let string_schema = StringSchema::construct(hash)?;
                    return Ok(Some(TypedSchema::String(string_schema)));
                }
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }
    Ok(None)
}

/// A Constructor constructs an object (a schema) from a saphyr::Hash
trait Constructor<T> {
    fn construct(hash: &saphyr::Hash) -> Result<T>;
}

fn load_string_value(value: &saphyr::Yaml) -> Result<String> {
    match value {
        saphyr::Yaml::String(s) => Ok(s.clone()),
        _ => Err(unsupported_type!(
            "Expected a string value, but got: {:?}",
            value
        )),
    }
}

impl Constructor<ArraySchema> for ArraySchema {
    fn construct(hash: &saphyr::Hash) -> Result<ArraySchema> {
        let mut array_schema = ArraySchema::default();
        for (key, value) in hash.iter() {
            if let saphyr::Yaml::String(key) = key {
                match key.as_str() {
                    "contains" => {
                        let yaml_schema = load_sub_schema(value.as_hash().unwrap())?.unwrap();
                        array_schema.contains = Some(Box::new(yaml_schema));
                    }
                    "items" => {
                        let array_items = load_array_items(value)?;
                        println!("array_items: {:#?}", array_items);
                        array_schema.items = Some(array_items);
                    }
                    "type" => {
                        let s = load_string_value(value)?;
                        if s != "array" {
                            return Err(unsupported_type!("Expected type: array, but got: {}", s));
                        }
                    }
                    "prefixItems" => {
                        if !value.is_array() {
                            return Err(unsupported_type!(
                                "Expected type: array, but got: {:?}",
                                value
                            ));
                        }
                        let prefix_items = load_prefix_items(value.as_vec().unwrap())?;
                        array_schema.prefix_items = Some(prefix_items);
                    }
                    _ => unimplemented!(),
                }
            }
        }
        Ok(array_schema)
    }
}

impl Constructor<IntegerSchema> for IntegerSchema {
    fn construct(hash: &saphyr::Hash) -> Result<IntegerSchema> {
        let mut integer_schema = IntegerSchema::default();
        for (key, value) in hash.iter() {
            if let saphyr::Yaml::String(key) = key {
                match key.as_str() {
                    "minimum" => {
                        integer_schema.minimum = Some(load_number(value)?);
                    }
                    "maximum" => {
                        integer_schema.maximum = Some(load_number(value)?);
                    }
                    "exclusiveMinimum" => {
                        integer_schema.exclusive_minimum = Some(load_number(value)?);
                    }
                    "exclusiveMaximum" => {
                        integer_schema.exclusive_maximum = Some(load_number(value)?);
                    }
                    "multipleOf" => {
                        integer_schema.multiple_of = Some(load_number(value)?);
                    }
                    "type" => {
                        let s = load_string_value(value)?;
                        if s != "integer" {
                            return Err(unsupported_type!("Expected type: integer, but got: {}", s));
                        }
                    }
                    _ => unimplemented!(),
                }
            }
        }
        Ok(integer_schema)
    }
}

impl Constructor<ObjectSchema> for ObjectSchema {
    fn construct(hash: &saphyr::Hash) -> Result<ObjectSchema> {
        let mut object_schema = ObjectSchema::default();
        for (key, value) in hash.iter() {
            if let saphyr::Yaml::String(key) = key {
                match key.as_str() {
                    "properties" => {
                        let properties = load_properties(value.as_hash().unwrap())?;
                        println!("properties: {:#?}", properties);
                        object_schema.properties = Some(properties);
                    }
                    "additionalProperties" => {
                        let additional_properties = load_additional_properties(value)?;
                        println!("additional_properties: {:#?}", additional_properties);
                        object_schema.additional_properties = Some(additional_properties);
                    }
                    "patternProperties" => {
                        let pattern_properties = load_properties(value.as_hash().unwrap())?;
                        println!("pattern_properties: {:#?}", pattern_properties);
                        object_schema.pattern_properties = Some(pattern_properties);
                    }
                    "type" => {
                        let s = load_string_value(value)?;
                        if s != "object" {
                            return Err(unsupported_type!("Expected type: object, but got: {}", s));
                        }
                    }
                    _ => unimplemented!(),
                }
            }
        }
        Ok(object_schema)
    }
}

fn load_properties(hash: &saphyr::Hash) -> Result<HashMap<String, YamlSchema>> {
    let mut properties = HashMap::new();
    for (key, value) in hash.iter() {
        if let saphyr::Yaml::String(key) = key {
            let schema = load_sub_schema(value.as_hash().unwrap())?.unwrap();
            properties.insert(key.clone(), schema);
        } else {
            return Err(unsupported_type!(
                "Expected a string key, but got: {:?}",
                key
            ));
        }
    }
    Ok(properties)
}

fn load_additional_properties(value: &saphyr::Yaml) -> Result<BoolOrTypedSchema> {
    match value {
        saphyr::Yaml::Boolean(b) => Ok(BoolOrTypedSchema::Boolean(*b)),
        saphyr::Yaml::Hash(hash) => {
            let schema = load_typed_schema(hash)?.unwrap();
            Ok(BoolOrTypedSchema::TypedSchema(Box::new(schema)))
        }
        _ => Err(unsupported_type!(
            "Expected type: boolean or hash, but got: {:?}",
            value
        )),
    }
}

fn load_number(value: &saphyr::Yaml) -> Result<Number> {
    match value {
        saphyr::Yaml::Integer(i) => Ok(Number::integer(*i)),
        saphyr::Yaml::Real(f) => Ok(Number::float(f.parse::<f64>()?)),
        _ => Err(unsupported_type!(
            "Expected type: integer or float, but got: {:?}",
            value
        )),
    }
}

impl Constructor<NumberSchema> for NumberSchema {
    fn construct(hash: &saphyr::Hash) -> Result<NumberSchema> {
        let mut number_schema = NumberSchema::default();
        for (key, value) in hash.iter() {
            if let saphyr::Yaml::String(key) = key {
                match key.as_str() {
                    "minimum" => {
                        let minimum = load_number(value).map_err(|_| {
                            crate::Error::UnsupportedType(format!(
                                "Expected type: integer or float, but got: {:?}",
                                &value
                            ))
                        })?;
                        number_schema.minimum = Some(minimum);
                    }
                    "maximum" => {
                        number_schema.maximum = Some(load_number(value)?);
                    }
                    "exclusiveMinimum" => {
                        number_schema.exclusive_minimum = Some(load_number(value)?);
                    }
                    "exclusiveMaximum" => {
                        number_schema.exclusive_maximum = Some(load_number(value)?);
                    }
                    "multipleOf" => {
                        number_schema.multiple_of = Some(load_number(value)?);
                    }
                    "type" => {
                        let s = load_string_value(value)?;
                        if s != "number" {
                            return Err(unsupported_type!("Expected type: number, but got: {}", s));
                        }
                    }
                    _ => unimplemented!(),
                }
            }
        }
        Ok(number_schema)
    }
}

impl Constructor<StringSchema> for StringSchema {
    fn construct(hash: &saphyr::Hash) -> Result<StringSchema> {
        let mut string_schema = StringSchema::default();
        for (key, value) in hash.iter() {
            if let saphyr::Yaml::String(key) = key {
                match key.as_str() {
                    "minLength" => {
                        if let saphyr::Yaml::Integer(i) = value {
                            string_schema.min_length = Some(*i as usize);
                        } else {
                            return Err(unsupported_type!(
                                "minLength expected integer, but got: {:?}",
                                value
                            ));
                        }
                    }
                    "maxLength" => {
                        if let saphyr::Yaml::Integer(i) = value {
                            string_schema.max_length = Some(*i as usize);
                        } else {
                            return Err(unsupported_type!(
                                "maxLength expected integer, but got: {:?}",
                                value
                            ));
                        }
                    }
                    "pattern" => {
                        if let saphyr::Yaml::String(s) = value {
                            let regex = regex::Regex::new(s.as_str())?;
                            string_schema.pattern = Some(regex);
                        } else {
                            return Err(unsupported_type!(
                                "pattern expected string, but got: {:?}",
                                value
                            ));
                        }
                    }
                    "type" => {
                        let s = load_string_value(value)?;
                        if s != "string" {
                            return Err(unsupported_type!("Expected type: string, but got: {}", s));
                        }
                    }
                    _ => unimplemented!(),
                }
            }
        }
        Ok(string_schema)
    }
}

impl Constructor<EnumSchema> for EnumSchema {
    fn construct(hash: &saphyr::Hash) -> Result<EnumSchema> {
        for (key, value) in hash.iter() {
            if let saphyr::Yaml::String(key) = key {
                match key.as_str() {
                    "enum" => {
                        let enum_values = load_enum_values(value.as_vec().unwrap())?;
                        return Ok(EnumSchema {
                            r#enum: enum_values,
                        });
                    }
                    _ => unimplemented!(),
                }
            }
        }
        unimplemented!()
    }
}

fn load_array_items(value: &saphyr::Yaml) -> Result<BoolOrTypedSchema> {
    match value {
        saphyr::Yaml::Boolean(b) => Ok(BoolOrTypedSchema::Boolean(*b)),
        saphyr::Yaml::Hash(hash) => Ok(BoolOrTypedSchema::TypedSchema(Box::new(
            load_typed_schema(hash)?.unwrap(),
        ))),
        _ => unimplemented!(),
    }
}

fn load_prefix_items(values: &Vec<saphyr::Yaml>) -> Result<Vec<Box<YamlSchema>>> {
    values
        .iter()
        .map(|v| {
            if !v.is_hash() {
                return Err(unsupported_type!("Expected type: hash, but got: {:?}", v));
            }
            let hash = v.as_hash().unwrap();
            let sub_schema = match load_sub_schema(hash) {
                Ok(Some(sub_schema)) => sub_schema,
                Ok(None) => return Err(unsupported_type!("Expected type: hash, but got: {:?}", v)),
                Err(e) => return Err(e),
            };
            Ok(Box::new(sub_schema))
        })
        .collect()
}

fn load_enum_values(values: &Vec<saphyr::Yaml>) -> Result<Vec<ConstValue>> {
    Ok(values
        .iter()
        .map(|v| ConstValue::from_saphyr_yaml(v))
        .collect())
}

/// Convert a Loader to a RootSchema
/// Just sets the schema to a YamlSchema::Empty if the loader schema is None
impl From<RootLoader> for RootSchema {
    fn from(loader: RootLoader) -> Self {
        RootSchema {
            id: loader.id,
            meta_schema: loader.meta_schema,
            schema: loader.schema.unwrap_or(YamlSchema::Empty),
        }
    }
}

fn yaml_to_string(yaml: &saphyr::Yaml, msg: &str) -> Result<String> {
    match yaml {
        saphyr::Yaml::String(s) => Ok(s.clone()),
        _ => Err(unsupported_type!("Expected a string, but got: {:?}", msg)),
    }
}

/// Convenience function to create saphyr::Yaml::String from a &str
fn sys(str: &str) -> saphyr::Yaml {
    saphyr::Yaml::String(String::from(str))
}

#[cfg(test)]
mod tests {

    use regex::Regex;

    use super::*;

    #[test]
    fn test_boolean_literal_true() {
        let root_schema = load_from_doc(&sys("true")).unwrap();
        assert_eq!(root_schema.schema, YamlSchema::BooleanLiteral(true));
    }

    #[test]
    fn test_boolean_literal_false() {
        let root_schema = load_from_doc(&sys("false")).unwrap();
        assert_eq!(root_schema.schema, YamlSchema::BooleanLiteral(false));
    }

    #[test]
    fn test_const_string() {
        let docs = saphyr::Yaml::load_from_str("const: string value").unwrap();
        let root_schema = load_from_doc(docs.first().unwrap()).unwrap();
        let const_schema = ConstSchema {
            r#const: ConstValue::string("string value"),
        };
        assert_eq!(root_schema.schema, YamlSchema::Const(const_schema));
    }

    #[test]
    fn test_const_integer() {
        let docs = saphyr::Yaml::load_from_str("const: 42").unwrap();
        let root_schema = load_from_doc(docs.first().unwrap()).unwrap();
        let const_schema = ConstSchema {
            r#const: ConstValue::integer(42),
        };
        assert_eq!(root_schema.schema, YamlSchema::Const(const_schema));
    }

    #[test]
    fn test_type_foo_should_error() {
        let docs = saphyr::Yaml::load_from_str("type: foo").unwrap();
        let root_schema = load_from_doc(docs.first().unwrap());
        assert!(root_schema.is_err());
        assert_eq!(
            root_schema.unwrap_err().to_string(),
            "Unsupported type 'foo'!"
        );
    }

    #[test]
    fn test_type_string() {
        let docs = saphyr::Yaml::load_from_str("type: string").unwrap();
        let root_schema = load_from_doc(docs.first().unwrap()).unwrap();
        let string_schema = StringSchema::default();
        assert_eq!(root_schema.schema, YamlSchema::String(string_schema));
    }

    #[test]
    fn test_type_string_with_pattern() {
        let docs = saphyr::Yaml::load_from_str(
            r#"
        type: string
        pattern: "^(\\([0-9]{3}\\))?[0-9]{3}-[0-9]{4}$"
        "#,
        )
        .unwrap();
        let root_schema = load_from_doc(docs.first().unwrap()).unwrap();
        let string_schema = StringSchema {
            pattern: Some(Regex::new("^(\\([0-9]{3}\\))?[0-9]{3}-[0-9]{4}$").unwrap()),
            ..Default::default()
        };
        assert_eq!(root_schema.schema, YamlSchema::String(string_schema));
    }

    #[test]
    fn test_array_constructor_items_true() {
        let mut hash = saphyr::Hash::new();
        hash.insert(sys("type"), sys("array"));
        hash.insert(sys("items"), saphyr::Yaml::Boolean(true));
        let array_schema = ArraySchema::construct(&hash).unwrap();
        assert_eq!(
            array_schema,
            ArraySchema {
                items: Some(BoolOrTypedSchema::Boolean(true)),
                prefix_items: None,
                contains: None
            }
        );
    }

    #[test]
    fn test_integer_schema() {
        let docs = saphyr::Yaml::load_from_str("type: integer").unwrap();
        let root_schema = load_from_doc(docs.first().unwrap()).unwrap();
        let integer_schema = IntegerSchema::default();
        assert_eq!(root_schema.schema, YamlSchema::Integer(integer_schema));
    }

    #[test]
    fn test_enum() {
        let docs = saphyr::Yaml::load_from_str(
            r#"
        enum:
          - foo
          - bar
          - baz
        "#,
        )
        .unwrap();
        let root_schema = load_from_doc(docs.first().unwrap()).unwrap();
        let enum_values = vec!["foo", "bar", "baz"]
            .iter()
            .map(|s| ConstValue::string(s.to_string()))
            .collect();
        let enum_schema = EnumSchema {
            r#enum: enum_values,
        };
        assert_eq!(root_schema.schema, YamlSchema::Enum(enum_schema));
    }

    #[test]
    fn test_enum_without_type() {
        let docs = saphyr::Yaml::load_from_str(
            r#"
            enum:
              - red
              - amber
              - green
              - null
              - 42
            "#,
        )
        .unwrap();
        let root_schema = load_from_doc(docs.first().unwrap()).unwrap();
        let enum_values = vec![
            ConstValue::string("red".to_string()),
            ConstValue::string("amber".to_string()),
            ConstValue::string("green".to_string()),
            ConstValue::null(),
            ConstValue::integer(42),
        ];
        let enum_schema = EnumSchema {
            r#enum: enum_values,
        };
        assert_eq!(root_schema.schema, YamlSchema::Enum(enum_schema));
    }
}
