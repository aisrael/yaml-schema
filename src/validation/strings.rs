use regex::Regex;

use crate::Context;
use crate::Result;
use crate::StringSchema;

use super::Validator;

impl Validator for StringSchema {
    fn validate(&self, context: &Context, value: &saphyr::Yaml) -> Result<()> {
        let errors = validate_string(
            self.min_length,
            self.max_length,
            self.pattern.as_ref(),
            value,
        );
        if !errors.is_empty() {
            for error in errors {
                context.add_error(error);
            }
        }
        Ok(())
    }
}

/// Just trying to isolate the actual validation into a function that doesn't take a context
pub fn validate_string(
    min_length: Option<usize>,
    max_length: Option<usize>,
    pattern: Option<&Regex>,
    value: &saphyr::Yaml,
) -> Vec<String> {
    let mut errors = Vec::new();
    let yaml_string = match value.as_str() {
        Some(s) => s,
        None => {
            errors.push(format!("Expected a string, but got: {:?}", value));
            return errors;
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
    errors
}

#[cfg(test)]
mod tests {
    use crate::Engine;
    use crate::RootSchema;
    use crate::YamlSchema;

    use super::*;

    #[test]
    fn test_engine_validate_string() {
        let schema = StringSchema::default();
        let root_schema = RootSchema::new(YamlSchema::String(schema));
        let context = Engine::evaluate(&root_schema, "some string", false).unwrap();
        assert!(!context.has_errors());
    }

    #[test]
    fn test_engine_validate_string_with_min_length() {
        let schema = StringSchema {
            min_length: Some(5),
            ..Default::default()
        };
        let root_schema = RootSchema::new(YamlSchema::String(schema));
        let context = Engine::evaluate(&root_schema, "hello", false).unwrap();
        assert!(!context.has_errors());
        let context = Engine::evaluate(&root_schema, "hell", false).unwrap();
        assert!(context.has_errors());
    }

    #[test]
    fn test_validate_string() {
        let errors = validate_string(None, None, None, &saphyr::Yaml::String("hello".to_string()));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_string_with_min_length() {
        let errors = validate_string(
            Some(5),
            None,
            None,
            &saphyr::Yaml::String("hello".to_string()),
        );
        assert!(errors.is_empty());
        let errors = validate_string(
            Some(5),
            None,
            None,
            &saphyr::Yaml::String("hell".to_string()),
        );
        assert!(!errors.is_empty());
        let first = errors.first().unwrap();
        assert_eq!(first, "String is too short! (min length: 5)");
    }

    #[test]
    fn test_string_schema_validation() {
        let schema = StringSchema::default();
        let value = saphyr::Yaml::String("Washington".to_string());
        let context = Context::default();
        let result = schema.validate(&context, &value);
        assert!(result.is_ok());
    }
}
