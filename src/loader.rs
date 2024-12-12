/// Loader defines the interface for loading a schema from a file or a string
use std::fs;

use crate::schemas::TypedSchema;
use crate::ArraySchema;
use crate::BoolOrTypedSchema;
use crate::ConstSchema;
use crate::ConstValue;
use crate::Error;
use crate::Number;
use crate::NumberSchema;
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
            saphyr::Yaml::String(s) => match s.as_str() {
                "array" => {
                    let array_schema = ArraySchema::construct(hash)?;
                    return Ok(Some(YamlSchema::Array(array_schema)));
                }
                "boolean" => return Ok(Some(YamlSchema::BooleanSchema)),
                "number" => {
                    let number_schema = NumberSchema::construct(hash)?;
                    return Ok(Some(YamlSchema::Number(number_schema)));
                }
                "string" => {
                    let string_schema = StringSchema::construct(hash)?;
                    return Ok(Some(YamlSchema::String(string_schema)));
                }
                _ => return unsupported_type!(s.to_string()),
            },
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

impl Constructor<ArraySchema> for ArraySchema {
    fn construct(hash: &saphyr::Hash) -> Result<ArraySchema> {
        let mut array_schema = ArraySchema::default();
        for (key, value) in hash.iter() {
            if let saphyr::Yaml::String(key) = key {
                match key.as_str() {
                    "items" => {
                        let array_items = load_array_items(value)?;
                        println!("array_items: {:#?}", array_items);
                        array_schema.items = Some(array_items);
                    }
                    "type" => {
                        if let saphyr::Yaml::String(s) = value {
                            if s != "array" {
                                return unsupported_type!("Expected type: array, but got: {}", s);
                            }
                        } else {
                            return unsupported_type!("Expected type: array, but got: {:?}", value);
                        }
                    }
                    "prefixItems" => {
                        if !value.is_array() {
                            return unsupported_type!("Expected type: array, but got: {:?}", value);
                        }
                        let prefix_items = load_prefix_items(value.as_vec().unwrap())?;
                        println!("prefix_items: {:#?}", prefix_items);
                        array_schema.prefix_items = Some(prefix_items);
                    }
                    _ => unimplemented!(),
                }
            }
        }
        Ok(array_schema)
    }
}

impl Constructor<NumberSchema> for NumberSchema {
    fn construct(hash: &saphyr::Hash) -> Result<NumberSchema> {
        let mut number_schema = NumberSchema::default();
        for (key, value) in hash.iter() {
            if let saphyr::Yaml::String(key) = key {
                match key.as_str() {
                    "minimum" => {
                        if let saphyr::Yaml::Integer(i) = value {
                            number_schema.minimum = Some(Number::integer(*i));
                        } else if let saphyr::Yaml::Real(f) = value {
                            number_schema.minimum = Some(Number::float(f.parse::<f64>()?));
                        } else {
                            return unsupported_type!(
                                "minimum expected integer or float, but got: {:?}",
                                value
                            );
                        }
                    }
                    "maximum" => {
                        if let saphyr::Yaml::Integer(i) = value {
                            number_schema.maximum = Some(Number::integer(*i));
                        } else if let saphyr::Yaml::Real(f) = value {
                            number_schema.maximum = Some(Number::float(f.parse::<f64>()?));
                        } else {
                            return unsupported_type!(
                                "maximum expected integer or float, but got: {:?}",
                                value
                            );
                        }
                    }
                    "exclusiveMinimum" => {
                        if let saphyr::Yaml::Integer(i) = value {
                            number_schema.exclusive_minimum = Some(Number::integer(*i));
                        } else if let saphyr::Yaml::Real(f) = value {
                            number_schema.exclusive_minimum = Some(Number::float(f.parse::<f64>()?));
                        } else {
                            return unsupported_type!(
                                "exclusiveMinimum expected integer or float, but got: {:?}",
                                value
                            );
                        }
                    }
                    "exclusiveMaximum" => {
                        if let saphyr::Yaml::Integer(i) = value {
                            number_schema.exclusive_maximum = Some(Number::integer(*i));
                        } else if let saphyr::Yaml::Real(f) = value {
                            number_schema.exclusive_maximum = Some(Number::float(f.parse::<f64>()?));
                        } else {
                            return unsupported_type!(
                                "exclusiveMaximum expected integer or float, but got: {:?}",
                                value
                            );
                        }
                    }
                    "multipleOf" => {
                        if let saphyr::Yaml::Integer(i) = value {
                            number_schema.multiple_of = Some(Number::integer(*i));
                        } else if let saphyr::Yaml::Real(f) = value {
                            number_schema.multiple_of = Some(Number::float(f.parse::<f64>()?));
                        } else {
                            return unsupported_type!(
                                "multipleOf expected integer or float, but got: {:?}",
                                value
                            );
                        }
                    }
                    "type" => {
                        if let saphyr::Yaml::String(s) = value {
                            if s != "number" {
                                return unsupported_type!("Expected type: number, but got: {}", s);
                            }
                        } else {
                            return unsupported_type!("Expected type: number, but got: {:?}", value);
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
                            return unsupported_type!(
                                "minLength expected integer, but got: {:?}",
                                value
                            );
                        }
                    }
                    "maxLength" => {
                        if let saphyr::Yaml::Integer(i) = value {
                            string_schema.max_length = Some(*i as usize);
                        } else {
                            return unsupported_type!(
                                "maxLength expected integer, but got: {:?}",
                                value
                            );
                        }
                    }
                    "pattern" => {
                        if let saphyr::Yaml::String(s) = value {
                            let regex = regex::Regex::new(s.as_str())?;
                            string_schema.pattern = Some(regex);
                        } else {
                            return unsupported_type!(
                                "pattern expected string, but got: {:?}",
                                value
                            );
                        }
                    }
                    "type" => {
                        if let saphyr::Yaml::String(s) = value {
                            if s != "string" {
                                return unsupported_type!("Expected type: string, but got: {}", s);
                            }
                        } else {
                            return unsupported_type!(
                                "Expected type: string, but got: {:?}",
                                value
                            );
                        }
                    }
                    _ => unimplemented!(),
                }
            }
        }
        Ok(string_schema)
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
        .map(|v| Ok(Box::new(load_sub_schema(v.as_hash().unwrap())?.unwrap())))
        .collect()
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
        _ => unsupported_type!("Expected a string, but got: {:?}", msg),
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
        let root_schema = load_from_doc(&docs.first().unwrap()).unwrap();
        let const_schema = ConstSchema {
            r#const: ConstValue::string("string value"),
        };
        assert_eq!(root_schema.schema, YamlSchema::Const(const_schema));
    }

    #[test]
    fn test_const_integer() {
        let docs = saphyr::Yaml::load_from_str("const: 42").unwrap();
        let root_schema = load_from_doc(&docs.first().unwrap()).unwrap();
        let const_schema = ConstSchema {
            r#const: ConstValue::integer(42),
        };
        assert_eq!(root_schema.schema, YamlSchema::Const(const_schema));
    }

    #[test]
    fn test_type_foo_should_error() {
        let docs = saphyr::Yaml::load_from_str("type: foo").unwrap();
        let root_schema = load_from_doc(&docs.first().unwrap());
        assert!(root_schema.is_err());
        assert_eq!(
            root_schema.unwrap_err().to_string(),
            "Unsupported type 'foo'!"
        );
    }

    #[test]
    fn test_type_string() {
        let docs = saphyr::Yaml::load_from_str("type: string").unwrap();
        let root_schema = load_from_doc(&docs.first().unwrap()).unwrap();
        let string_schema = StringSchema::default();
        assert_eq!(root_schema.schema, YamlSchema::String(string_schema));
    }

    #[test]
    fn test_type_string_with_pattern() {
        let docs = saphyr::Yaml::load_from_str(
 r#"
        type: string
        pattern: "^(\\([0-9]{3}\\))?[0-9]{3}-[0-9]{4}$"
        "#).unwrap();
        let root_schema = load_from_doc(&docs.first().unwrap()).unwrap();
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
}
