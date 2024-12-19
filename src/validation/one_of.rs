use log::{debug, error};

use super::Validator;
use crate::Context;
use crate::Error;
use crate::Result;
use crate::YamlSchema;

impl Validator for crate::schemas::OneOfSchema {
    fn validate(&self, context: &Context, value: &saphyr::MarkedYaml) -> Result<()> {
        let one_of_is_valid = validate_one_of(context, &self.one_of, value)?;
        if !one_of_is_valid {
            error!("OneOf: None of the schemas in `oneOf` matched!");
            context.add_error(value, "None of the schemas in `oneOf` matched!");
            fail_fast!(context);
        }
        Ok(())
    }
}

pub fn validate_one_of(
    context: &Context,
    schemas: &Vec<YamlSchema>,
    value: &saphyr::MarkedYaml,
) -> Result<bool> {
    let mut one_of_is_valid = false;
    for schema in schemas {
        debug!(
            "OneOf: Validating value: {:?} against schema: {}",
            value, schema
        );
        let sub_context = Context::new(true);
        let sub_result = schema.validate(&sub_context, value);
        match sub_result {
            Ok(()) | Err(Error::FailFast) => {
                debug!(
                    "OneOf: sub_context.errors: {}",
                    sub_context.errors.borrow().len()
                );
                if sub_context.has_errors() {
                    continue;
                }

                if one_of_is_valid {
                    error!("OneOf: Value matched multiple schemas in `oneOf`!");
                    context.add_error(value, "Value matched multiple schemas in `oneOf`!");
                    fail_fast!(context);
                } else {
                    one_of_is_valid = true;
                }
            }
            Err(e) => return Err(e),
        }
    }
    debug!("OneOf: one_of_is_valid: {}", one_of_is_valid);
    Ok(one_of_is_valid)
}
