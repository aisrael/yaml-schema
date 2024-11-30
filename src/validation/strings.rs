use regex::Regex;

use crate::Result;

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
