use log::debug;

use crate::format_vec;
use crate::Result;
use crate::Validator;
use crate::YamlSchema;

use super::BoolOrTypedSchema;

/// An array schema represents an array
#[derive(Debug, Default, PartialEq)]
pub struct ArraySchema {
    pub items: Option<BoolOrTypedSchema>,
    pub prefix_items: Option<Vec<Box<YamlSchema>>>,
    pub contains: Option<Box<YamlSchema>>,
}

impl std::fmt::Display for ArraySchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Array {:?}", self)
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
                BoolOrTypedSchema::Boolean(true) => { /* no-op */ }
                BoolOrTypedSchema::Boolean(false) => {
                    if self.prefix_items.is_none() {
                        context.add_error("Array items are not allowed!".to_string());
                    }
                }
                BoolOrTypedSchema::TypedSchema(typed_schema) => {
                    for item in array {
                        typed_schema.validate(context, item)?;
                    }
                }
                BoolOrTypedSchema::MultipleTypeNames(types) => {
                    unimplemented!(
                        "Array items with multiple types not yet supported: {}",
                        format_vec(types)
                    )
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
                        BoolOrTypedSchema::Boolean(true) => {
                            // `items: true` allows any items
                            break;
                        }
                        BoolOrTypedSchema::Boolean(false) => {
                            context
                                .add_error("Additional array items are not allowed!".to_string());
                        }
                        BoolOrTypedSchema::TypedSchema(typed_schema) => {
                            typed_schema.validate(context, item)?;
                        }
                        BoolOrTypedSchema::MultipleTypeNames(types) => {
                            unimplemented!(
                                "Array items with multiple types not yet supported: {}",
                                format_vec(types)
                            )
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
