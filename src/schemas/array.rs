use std::fmt;

use super::BoolOrTypedSchema;
use crate::{deser_typed_schema, YamlSchema};

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

impl From<&crate::deser::TypedSchema> for ArraySchema {
    fn from(t: &crate::deser::TypedSchema) -> Self {
        let items: Option<BoolOrTypedSchema> = t.items.as_ref().map(|i| match i {
            crate::deser::ArrayItemsValue::Boolean(b) => BoolOrTypedSchema::Boolean(*b),
            crate::deser::ArrayItemsValue::TypedSchema(t) => {
                BoolOrTypedSchema::TypedSchema(Box::new(deser_typed_schema(t)))
            }
        });
        let prefix_items: Option<Vec<Box<YamlSchema>>> =
            t.prefix_items.as_ref().map(|prefix_items| {
                prefix_items
                    .iter()
                    .map(|y: &crate::deser::YamlSchema| Box::new(y.into()))
                    .collect()
            });
        let contains: Option<Box<YamlSchema>> = t.contains.as_ref().map(|c| Box::new(c.into()));
        ArraySchema {
            items,
            prefix_items,
            contains,
        }
    }
}
