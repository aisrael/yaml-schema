use std::collections::HashMap;
use std::fs;

use crate::schemas::TypedSchema;
use crate::AnyOfSchema;
use crate::ArraySchema;
use crate::BoolOrTypedSchema;
use crate::ConstSchema;
use crate::ConstValue;
use crate::EnumSchema;
use crate::Error;
use crate::IntegerSchema;
use crate::NotSchema;
use crate::Number;
use crate::NumberSchema;
use crate::ObjectSchema;
use crate::OneOfSchema;
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
        self.schema = Some(YamlSchema::construct(hash)?);
        Ok(())
    }
}

impl Constructor<YamlSchema> for YamlSchema {
    fn construct(hash: &saphyr::Hash) -> Result<YamlSchema> {
        if hash.contains_key(&sys("type")) {
            match TypedSchema::construct(hash) {
                Ok(typed_schema) => Ok(typed_schema.into()),
                Err(e) => Err(e),
            }
        } else if hash.contains_key(&sys("enum")) {
            let enum_schema = EnumSchema::construct(hash)?;
            return Ok(YamlSchema::Enum(enum_schema));
        } else if hash.contains_key(&sys("const")) {
            let const_schema = ConstSchema::construct(hash)?;
            return Ok(YamlSchema::Const(const_schema));
        } else if hash.contains_key(&sys("anyOf")) {
            let any_of_schema = AnyOfSchema::construct(hash)?;
            return Ok(YamlSchema::AnyOf(any_of_schema));
        } else if hash.contains_key(&sys("oneOf")) {
            let one_of_schema = OneOfSchema::construct(hash)?;
            return Ok(YamlSchema::OneOf(one_of_schema));
        } else if hash.contains_key(&sys("not")) {
            let not_schema = NotSchema::construct(hash)?;
            return Ok(YamlSchema::Not(not_schema));
        } else {
            unimplemented!()
        }
    }
}

impl Constructor<TypedSchema> for TypedSchema {
    fn construct(hash: &saphyr::Hash) -> Result<TypedSchema> {
        let type_key = saphyr::Yaml::String(String::from("type"));
        if hash.contains_key(&type_key) {
            let value = hash.get(&type_key).unwrap();
            match value {
                saphyr::Yaml::String(s) => match s.as_str() {
                    "array" => {
                        let array_schema = ArraySchema::construct(hash)?;
                        Ok(TypedSchema::Array(array_schema))
                    }
                    "boolean" => Ok(TypedSchema::BooleanSchema),
                    "integer" => {
                        let integer_schema = IntegerSchema::construct(hash)?;
                        Ok(TypedSchema::Integer(integer_schema))
                    }
                    "number" => {
                        let number_schema = NumberSchema::construct(hash)?;
                        Ok(TypedSchema::Number(number_schema))
                    }
                    "object" => {
                        let object_schema = ObjectSchema::construct(hash)?;
                        Ok(TypedSchema::Object(object_schema))
                    }
                    "string" => {
                        let string_schema = StringSchema::construct(hash)?;
                        Ok(TypedSchema::String(string_schema))
                    }
                    s => Err(unsupported_type!(s.to_string())),
                },
                saphyr::Yaml::Null => Ok(TypedSchema::Null),
                v => Err(unsupported_type!(
                    "Expected type: string, but got: {:#?}",
                    v
                )),
            }
        } else {
            generic_error!("No type key found in hash: {:#?}", hash)
        }
    }
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
                        let yaml_schema = YamlSchema::construct(value.as_hash().unwrap())?;
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
                        let prefix_items = load_array_of_schemas(value)?;
                        array_schema.prefix_items = Some(prefix_items);
                    }
                    _ => unimplemented!(),
                }
            }
        }
        Ok(array_schema)
    }
}

impl Constructor<ConstSchema> for ConstSchema {
    fn construct(hash: &saphyr::Hash) -> Result<ConstSchema> {
        let const_key = sys("const");
        let value = hash.get(&const_key).unwrap();
        match value {
            saphyr::Yaml::String(s) => Ok(ConstSchema {
                r#const: ConstValue::string(s),
            }),
            saphyr::Yaml::Integer(i) => Ok(ConstSchema {
                r#const: ConstValue::integer(*i),
            }),
            saphyr::Yaml::Real(s) => {
                let f = s.parse::<f64>()?;
                Ok(ConstSchema {
                    r#const: ConstValue::float(f),
                })
            }
            _ => unimplemented!(),
        }
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
                            return Err(unsupported_type!(
                                "Expected type: integer, but got: {}",
                                s
                            ));
                        }
                    }
                    _ => unimplemented!(),
                }
            }
        }
        Ok(integer_schema)
    }
}

impl Constructor<EnumSchema> for EnumSchema {
    fn construct(hash: &saphyr::Hash) -> Result<EnumSchema> {
        let enum_key = sys("enum");
        let value = hash.get(&enum_key).unwrap();
        match value {
            saphyr::Yaml::Array(values) => {
                let enum_values = load_enum_values(values)?;
                Ok(EnumSchema {
                    r#enum: enum_values,
                })
            }
            v => generic_error!("enum: Expected an array, but got: {:#?}", v),
        }
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
                    "minProperties" => {
                        object_schema.min_properties = Some(load_integer(value)? as usize);
                    }
                    "maxProperties" => {
                        object_schema.max_properties = Some(load_integer(value)? as usize);
                    }
                    "patternProperties" => {
                        let pattern_properties = load_properties(value.as_hash().unwrap())?;
                        println!("pattern_properties: {:#?}", pattern_properties);
                        object_schema.pattern_properties = Some(pattern_properties);
                    }
                    "propertyNames" => {
                        if !value.is_hash() {
                            return Err(unsupported_type!(
                                "propertyNames: Expected a hash, but got: {:?}",
                                value
                            ));
                        }
                        let hash = value.as_hash().unwrap();
                        if !hash.contains_key(&sys("pattern")) {
                            return Err(generic_error!(
                                "propertyNames: Missing required key: pattern"
                            ));
                        }
                        let pattern = load_string_value(hash.get(&sys("pattern")).unwrap())?;
                        object_schema.property_names = Some(pattern);
                    }
                    "required" => {
                        if !value.is_array() {
                            return Err(unsupported_type!(
                                "required: Expected an array, but got: {:?}",
                                value
                            ));
                        }
                        let array = value.as_vec().unwrap();
                        object_schema.required = Some(
                            array
                                .iter()
                                .map(|v| match v {
                                    saphyr::Yaml::String(s) => Ok(s.clone()),
                                    _ => unimplemented!(),
                                })
                                .collect::<Result<Vec<String>>>()?,
                        );
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

fn load_array_of_schemas(value: &saphyr::Yaml) -> Result<Vec<YamlSchema>> {
    match value {
        saphyr::Yaml::Array(values) => values
            .iter()
            .map(|v| match v {
                saphyr::Yaml::Hash(hash) => YamlSchema::construct(hash),
                _ => unimplemented!(),
            })
            .collect::<Result<Vec<YamlSchema>>>(),
        _ => unimplemented!(),
    }
}

impl Constructor<AnyOfSchema> for AnyOfSchema {
    fn construct(hash: &saphyr::Hash) -> Result<AnyOfSchema> {
        let mut any_of_schema = AnyOfSchema::default();
        for (key, value) in hash.iter() {
            if let saphyr::Yaml::String(key) = key {
                match key.as_str() {
                    "anyOf" => {
                        any_of_schema.any_of = load_array_of_schemas(value)?;
                    }
                    _ => unimplemented!(),
                }
            }
        }
        Ok(any_of_schema)
    }
}

impl Constructor<OneOfSchema> for OneOfSchema {
    fn construct(hash: &saphyr::Hash) -> Result<OneOfSchema> {
        let mut one_of_schema = OneOfSchema::default();
        for (key, value) in hash.iter() {
            if let saphyr::Yaml::String(key) = key {
                match key.as_str() {
                    "oneOf" => {
                        one_of_schema.one_of = load_array_of_schemas(value)?;
                    }
                    _ => unimplemented!(),
                }
            }
        }
        Ok(one_of_schema)
    }
}

fn load_properties(hash: &saphyr::Hash) -> Result<HashMap<String, YamlSchema>> {
    let mut properties = HashMap::new();
    for (key, value) in hash.iter() {
        if let saphyr::Yaml::String(key) = key {
            let schema = YamlSchema::construct(value.as_hash().unwrap())?;
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
            let schema = TypedSchema::construct(hash)?;
            Ok(BoolOrTypedSchema::TypedSchema(Box::new(schema)))
        }
        _ => Err(unsupported_type!(
            "Expected type: boolean or hash, but got: {:?}",
            value
        )),
    }
}

impl Constructor<NotSchema> for NotSchema {
    fn construct(hash: &saphyr::Hash) -> Result<NotSchema> {
        for (key, value) in hash.iter() {
            if let saphyr::Yaml::String(key) = key {
                match key.as_str() {
                    "not" => {
                        let schema = YamlSchema::construct(value.as_hash().unwrap())?;
                        return Ok(NotSchema {
                            not: Box::new(schema),
                        });
                    }
                    _ => unimplemented!(),
                }
            }
        }
        unimplemented!()
    }
}

fn load_integer(value: &saphyr::Yaml) -> Result<i64> {
    match value {
        saphyr::Yaml::Integer(i) => Ok(*i),
        _ => Err(unsupported_type!(
            "Expected type: integer, but got: {:?}",
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

fn load_array_items(value: &saphyr::Yaml) -> Result<BoolOrTypedSchema> {
    match value {
        saphyr::Yaml::Boolean(b) => Ok(BoolOrTypedSchema::Boolean(*b)),
        saphyr::Yaml::Hash(hash) => Ok(BoolOrTypedSchema::TypedSchema(Box::new(
            TypedSchema::construct(hash)?,
        ))),
        _ => unimplemented!(),
    }
}

fn load_enum_values(values: &Vec<saphyr::Yaml>) -> Result<Vec<ConstValue>> {
    Ok(values.iter().map(ConstValue::from_saphyr_yaml).collect())
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
        let enum_values = ["foo", "bar", "baz"]
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
