use log::debug;

use super::validation::Validator;
pub use crate::validation::Context;
pub use crate::validation::ValidationError;
use crate::Error;
use crate::Result;
use crate::YamlSchema;

pub struct Engine<'a> {
    pub schema: &'a YamlSchema,
}

impl<'a> Engine<'a> {
    pub fn new(schema: &'a YamlSchema) -> Engine<'a> {
        Engine { schema }
    }

    pub fn evaluate(&self, yaml: &serde_yaml::Value, fail_fast: bool) -> Result<Context> {
        debug!("Engine is running");
        let context = Context::new(fail_fast);
        let result = self.schema.validate(&context, yaml);
        debug!("Engine: result: {:?}", result);
        debug!("Engine: context.errors: {}", context.errors.borrow().len());
        match result {
            Ok(()) | Err(Error::FailFast) => Ok(context),
            Err(e) => Err(e),
        }
    }
}

// impl TypedSchema {
//     fn validate_boolean(
//         &self,
//         context: &Context,
//         value: &serde_yaml::Value,
//     ) -> Result<()> {
//         if !value.is_bool() {
//             context.add_error(format!("Expected a boolean, but got: {:?}", value));
//             fail_fast!(context);
//         }
//         Ok(())
//     }

//     fn validate_integer(&self, context: &Context, value: &serde_yaml::Value) {
//         match value.as_i64() {
//             Some(i) => self.validate_number_i64(context, i),
//             None => {
//                 if value.is_f64() {
//                     let f = value.as_f64().unwrap();
//                     if f.fract() == 0.0 {
//                         return self.validate_number_i64(context, f as i64);
//                     } else {
//                         context.add_error(format!("Expected an integer, but got: {:?}", value));
//                     }
//                 }
//                 context.add_error(format!("Expected an integer, but got: {:?}", value));
//             }
//         }
//     }

//     fn validate_number(&self, context: &Context, value: &serde_yaml::Value) {
//         if value.is_i64() {
//             match value.as_i64() {
//                 Some(i) => self.validate_number_i64(context, i),
//                 None => {
//                     context.add_error(format!("Expected an integer, but got: {:?}", value));
//                 }
//             }
//         } else if value.is_f64() {
//             match value.as_f64() {
//                 Some(f) => self.validate_number_f64(context, f),
//                 None => {
//                     context.add_error(format!("Expected a float, but got: {:?}", value));
//                 }
//             }
//         } else {
//             context.add_error(format!("Expected a number, but got: {:?}", value));
//         }
//     }

//     fn validate_number_i64(&self, context: &Context, i: i64) {
//         if let Some(minimum) = &self.minimum {
//             match minimum {
//                 Number::Integer(min) => {
//                     if i < *min {
//                         context.add_error("Number is too small!".to_string());
//                     }
//                 }
//                 Number::Float(min) => {
//                     if (i as f64) < *min {
//                         context.add_error("Number is too small!".to_string());
//                     }
//                 }
//             }
//         }
//         if let Some(maximum) = &self.maximum {
//             match maximum {
//                 Number::Integer(max) => {
//                     if i > *max {
//                         context.add_error("Number is too big!".to_string());
//                     }
//                 }
//                 YamlSchemaNumber::Float(max) => {
//                     if (i as f64) > *max {
//                         context.add_error("Number is too big!".to_string());
//                     }
//                 }
//             }
//         }
//         if let Some(multiple_of) = &self.multiple_of {
//             match multiple_of {
//                 YamlSchemaNumber::Integer(multiple) => {
//                     if i % *multiple != 0 {
//                         context.add_error(format!("Number is not a multiple of {}!", multiple));
//                     }
//                 }
//                 YamlSchemaNumber::Float(multiple) => {
//                     if (i as f64) % *multiple != 0.0 {
//                         context.add_error(format!("Number is not a multiple of {}!", multiple));
//                     }
//                 }
//             }
//         }
//     }
//     fn validate_number_f64(&self, context: &Context, f: f64) {
//         if let Some(minimum) = &self.minimum {
//             match minimum {
//                 YamlSchemaNumber::Integer(min) => {
//                     if f < *min as f64 {
//                         context.add_error("Number is too small!".to_string());
//                     }
//                 }
//                 YamlSchemaNumber::Float(min) => {
//                     if f < *min {
//                         context.add_error("Number is too small!".to_string());
//                     }
//                 }
//             }
//         }
//         if let Some(maximum) = &self.maximum {
//             match maximum {
//                 YamlSchemaNumber::Integer(max) => {
//                     if f > *max as f64 {
//                         context.add_error("Number is too big!".to_string());
//                     }
//                 }
//                 YamlSchemaNumber::Float(max) => {
//                     if f > *max {
//                         context.add_error("Number is too big!".to_string());
//                     }
//                 }
//             }
//         }
//     }

//     /// Validate the string according to the schema rules
//     fn validate_string(
//         &self,
//         context: &Context,
//         value: &serde_yaml::Value,
//     ) -> Result<()> {
//         match validate_string(
//             self.min_length,
//             self.max_length,
//             self.pattern.as_ref(),
//             value,
//         ) {
//             Ok(errors) => {
//                 if !errors.is_empty() {
//                     for error in errors {
//                         debug!("validate_string: error: {}", error);
//                         context.add_error(error);
//                     }
//                     fail_fast!(context);
//                 }
//                 Ok(())
//             }
//             Err(e) => {
//                 let s = e.to_string();
//                 error!("{}", s);
//                 Err(Error::GenericError(s))
//             }
//         }
//     }

//     /// Validate the object according to the schema rules
//     fn validate_object(
//         &self,
//         context: &Context,
//         value: &serde_yaml::Value,
//     ) -> Result<()> {
//         debug!("Validating object: {:?}", value);
//         match value.as_mapping() {
//             Some(mapping) => self.validate_object_mapping(context, mapping),
//             None => {
//                 context.add_error("Expected an object, but got: None");
//                 Ok(())
//             }
//         }
//     }

//     fn validate_object_mapping(
//         &self,
//         context: &Context,
//         mapping: &serde_yaml::Mapping,
//     ) -> Result<()> {
//         for (k, value) in mapping {
//             let key = match k {
//                 serde_yaml::Value::String(s) => s.clone(),
//                 _ => k.as_str().unwrap_or_default().to_string(),
//             };
//             debug!("validate_object_mapping: key: \"{}\"", key);
//             // First, we check the explicitly defined properties, and validate against it if found
//             if let Some(properties) = &self.properties {
//                 if try_validate_value_against_properties(context, &key, value, properties)? {
//                     continue;
//                 }
//             }

//             // Then, we check if additional properties are allowed or not
//             if let Some(additional_properties) = &self.additional_properties {
//                 if !try_validate_value_against_additional_properties(
//                     context,
//                     &key,
//                     value,
//                     additional_properties,
//                 )? {
//                     return Ok(());
//                 }
//             }
//             // Then we check if pattern_properties matches
//             if let Some(pattern_properties) = &self.pattern_properties {
//                 for (pattern, schema) in pattern_properties {
//                     // TODO: compile the regex once instead of every time we're evaluating
//                     let re = regex::Regex::new(pattern).map_err(|e| {
//                         Error::GenericError(format!(
//                             "Invalid regular expression pattern: {}",
//                             e
//                         ))
//                     })?;
//                     if re.is_match(key.as_str()) {
//                         schema.validate(context, value)?;
//                     }
//                 }
//             }
//             // Finally, we check if it matches property_names
//             if let Some(property_names) = &self.property_names {
//                 let re = regex::Regex::new(&property_names.pattern).map_err(|e| {
//                     Error::GenericError(format!(
//                         "Invalid regular expression pattern: {}",
//                         e
//                     ))
//                 })?;
//                 debug!("Regex for property names: {}", re.as_str());
//                 if !re.is_match(key.as_str()) {
//                     return Err(Error::GenericError(format!(
//                         "Property name '{}' does not match pattern specified in `propertyNames`!",
//                         key
//                     )));
//                 }
//             }
//         }

//         // Validate required properties
//         if let Some(required) = &self.required {
//             for required_property in required {
//                 if !mapping.contains_key(required_property) {
//                     return Err(Error::GenericError(format!(
//                         "Required property '{}' is missing!",
//                         required_property
//                     )));
//                 }
//             }
//         }

//         // Validate minProperties
//         if let Some(min_properties) = &self.min_properties {
//             if mapping.len() < *min_properties {
//                 return Err(Error::GenericError(format!(
//                     "Object has too few properties! Minimum is {}!",
//                     min_properties
//                 )));
//             }
//         }
//         // Validate maxProperties
//         if let Some(max_properties) = &self.max_properties {
//             if mapping.len() > *max_properties {
//                 return Err(Error::GenericError(format!(
//                     "Object has too many properties! Maximum is {}!",
//                     max_properties
//                 )));
//             }
//         }

//         Ok(())
//     }

//     fn validate_array(
//         &self,
//         context: &Context,
//         value: &serde_yaml::Value,
//     ) -> Result<()> {
//         if !value.is_sequence() {
//             context.add_error(format!("Expected an array, but got: {:?}", value));
//             fail_fast!(context);
//             return Ok(());
//         }

//         let array = value.as_sequence().unwrap();

//         // validate array items
//         if let Some(items) = &self.items {
//             match items {
//                 ArrayItemsValue::TypedSchema(typed_schema) => {
//                     for item in array {
//                         typed_schema.validate(context, item)?;
//                     }
//                 }
//                 ArrayItemsValue::Boolean(true) => { /* no-op */ }
//                 ArrayItemsValue::Boolean(false) => {
//                     if self.prefix_items.is_none() {
//                         return Err(Error::GenericError(
//                             "Array items are not allowed!".to_string(),
//                         ));
//                     }
//                 }
//             }
//         }

//         // validate contains
//         if let Some(contains) = &self.contains {
//             if !array
//                 .iter()
//                 .any(|item| contains.validate(context, item).is_ok())
//             {
//                 return Err(Error::GenericError(
//                     "Contains validation failed!".to_string(),
//                 ));
//             }
//         }

//         // validate prefix items
//         if let Some(prefix_items) = &self.prefix_items {
//             debug!("Validating prefix items: {}", format_vec(prefix_items));
//             for (i, item) in array.iter().enumerate() {
//                 // if the index is within the prefix items, validate against the prefix items schema
//                 if i < prefix_items.len() {
//                     debug!(
//                         "Validating prefix item {} with schema: {}",
//                         i, prefix_items[i]
//                     );
//                     prefix_items[i].validate(context, item)?;
//                 } else if let Some(items) = &self.items {
//                     // if the index is not within the prefix items, validate against the array items schema
//                     match items {
//                         ArrayItemsValue::TypedSchema(typed_schema) => {
//                             typed_schema.validate(context, item)?;
//                         }
//                         ArrayItemsValue::Boolean(true) => {
//                             // `items: true` allows any items
//                             break;
//                         }
//                         ArrayItemsValue::Boolean(false) => {
//                             return Err(Error::GenericError(
//                                 "Additional array items are not allowed!".to_string(),
//                             ));
//                         }
//                     }
//                 } else {
//                     break;
//                 }
//             }
//         }

//         Ok(())
//     }
// }
