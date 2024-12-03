use crate::Context;
use crate::Result;
use crate::StringSchema;
use regex::Regex;

use super::Validator;

impl Validator for StringSchema {
    fn validate(&self, context: &Context, value: &serde_yaml::Value) -> Result<()> {
        match validate_string(
            self.min_length,
            self.max_length,
            self.pattern.as_ref(),
            value,
        ) {
            Ok(errors) => {
                if !errors.is_empty() {
                    for error in errors {
                        context.add_error(error);
                    }
                }
                Ok(())
            }
            Err(e) => generic_error!("{}", e),
        }
    }
}

/// Just trying to isolate the actual validation into a function that doesn't take a context
pub fn validate_string(
    min_length: Option<usize>,
    max_length: Option<usize>,
    pattern: Option<&Regex>,
    value: &serde_yaml::Value,
) -> Result<Vec<String>> {
    let mut errors = Vec::new();
    let yaml_string = match value.as_str() {
        Some(s) => s,
        None => {
            errors.push(format!("Expected a string, but got: {:?}", value));
            return Ok(errors);
        }
    };
    if let Some(min_length) = min_length {
        if yaml_string.len() < min_length {
            errors.push(format!("String is too short! (min length: {})", min_length));
        }
    }
    if let Some(max_length) = max_length {
        if yaml_string.len() > max_length {
            errors.push(format!("String is too long! (max length: {})", max_length));
        }
    }
    if let Some(regex) = pattern {
        if !regex.is_match(yaml_string) {
            errors.push(format!(
                "String does not match regular expression {}!",
                regex.as_str()
            ));
        }
    }
    Ok(errors)
}
