use log::debug;
use std::fmt;

use crate::deser::Deser;
use crate::deser_typed_schema;
use crate::format_vec;
use crate::Result;
use crate::Validator;
use crate::YamlSchema;

use super::BoolOrTypedSchema;

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
                    .map(|y: &crate::deser::YamlSchema| Box::new(y.deserialize().unwrap()))
                    .collect()
            });
        let contains: Option<Box<YamlSchema>> = t
            .contains
            .as_ref()
            .map(|c| Box::new(c.deserialize().unwrap()));
        ArraySchema {
            items,
            prefix_items,
            contains,
        }
    }
}

impl Validator for ArraySchema {
    fn validate(&self, context: &crate::Context, value: &serde_yaml::Value) -> Result<()> {
        if !value.is_sequence() {
            context.add_error(format!("Expected an array, but got: {:?}", value));
            fail_fast!(context);
            return Ok(());
        }

        let array = value.as_sequence().unwrap();

        // validate array items
        if let Some(items) = &self.items {
            match items {
                BoolOrTypedSchema::TypedSchema(typed_schema) => {
                    for item in array {
                        typed_schema.validate(context, item)?;
                    }
                }
                BoolOrTypedSchema::Boolean(true) => { /* no-op */ }
                BoolOrTypedSchema::Boolean(false) => {
                    if self.prefix_items.is_none() {
                        context.add_error("Array items are not allowed!".to_string());
                    }
                }
            }
        }

        // validate contains
        if let Some(contains) = &self.contains {
            if !array
                .iter()
                .any(|item| contains.validate(context, item).is_ok())
            {
                context.add_error("Contains validation failed!".to_string());
            }
        }

        // validate prefix items
        if let Some(prefix_items) = &self.prefix_items {
            debug!("Validating prefix items: {}", format_vec(prefix_items));
            for (i, item) in array.iter().enumerate() {
                // if the index is within the prefix items, validate against the prefix items schema
                if i < prefix_items.len() {
                    debug!(
                        "Validating prefix item {} with schema: {}",
                        i, prefix_items[i]
                    );
                    prefix_items[i].validate(context, item)?;
                } else if let Some(items) = &self.items {
                    // if the index is not within the prefix items, validate against the array items schema
                    match items {
                        BoolOrTypedSchema::TypedSchema(typed_schema) => {
                            typed_schema.validate(context, item)?;
                        }
                        BoolOrTypedSchema::Boolean(true) => {
                            // `items: true` allows any items
                            break;
                        }
                        BoolOrTypedSchema::Boolean(false) => {
                            context
                                .add_error("Additional array items are not allowed!".to_string());
                        }
                    }
                } else {
                    break;
                }
            }
        }

        Ok(())
    }
}
