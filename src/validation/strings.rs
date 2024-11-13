/// Just trying to isolate the actual validation into a function that doesn't take a context
pub fn validate_string(
    min_length: Option<usize>,
    max_length: Option<usize>,
    pattern: Option<&String>,
    value: &serde_yaml::Value,
) -> Result<Vec<String>, regex::Error> {
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
            errors.push("String is too short!".to_string());
        }
    }
    if let Some(max_length) = max_length {
        if yaml_string.len() > max_length {
            errors.push("String is too long!".to_string());
        }
    }
    if let Some(pattern) = pattern {
        let re = regex::Regex::new(pattern)?;
        if !re.is_match(yaml_string) {
            errors.push("String does not match regex!".to_string());
        }
    }
    Ok(errors)
}
