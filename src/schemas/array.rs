use std::fmt;

use super::BoolOrTypedSchema;
use crate::YamlSchema;

/// An array schema represents an array
#[derive(Debug, PartialEq)]
pub struct ArraySchema {
    pub items: Option<BoolOrTypedSchema>,
    pub prefix_items: Option<Vec<Box<YamlSchema>>>,
    pub contains: Option<Box<YamlSchema>>,
}

impl fmt::Display for ArraySchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Array {:?}", self)
    }
}
