use super::Validator;
use crate::{Context, Error, YamlSchema};
use log::{debug, error};

pub fn validate_one_of(
    context: &Context,
    schemas: &Vec<YamlSchema>,
    value: &serde_yaml::Value,
) -> Result<bool, Error> {
    let mut one_of_is_valid = false;
    for schema in schemas {
        debug!(
            "OneOf: Validating value: {:?} against schema: {}",
            value, schema
        );
        let sub_context = Context::new(context.fail_fast);
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
                    context.add_error("Value matched multiple schemas in `oneOf`!");
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
