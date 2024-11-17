use std::collections::HashMap;
use std::fmt;

use super::{BoolOrTypedSchema, TypedSchema};
use crate::YamlSchema;

/// An object schema
#[derive(Debug, PartialEq)]
pub struct ObjectSchema {
    pub properties: Option<HashMap<String, YamlSchema>>,
    pub required: Option<Vec<String>>,
    pub additional_properties: Option<BoolOrTypedSchema>,
    pub pattern_properties: Option<HashMap<String, YamlSchema>>,
    pub property_names: Option<PropertyNamesValue>,
    pub min_properties: Option<usize>,
    pub max_properties: Option<usize>,
}

#[derive(Debug, PartialEq)]
pub struct PropertyNamesValue {
    pub pattern: String,
}

pub enum AdditionalProperties {
    Boolean(bool),
    TypedSchema(Box<TypedSchema>),
}

impl fmt::Display for ObjectSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Object {:?}", self)
    }
}
