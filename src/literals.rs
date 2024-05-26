use serde::{Deserialize, Serialize};

use crate::error::YamlSchemaError;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Literal {
    String(YamlString),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct YamlString {
    pub max_length: Option<u64>,
    pub min_length: Option<u64>,
    pub pattern: Option<String>,
}

impl YamlString {
    pub fn validate(&self, value: &serde_yaml::Value) -> Result<(), YamlSchemaError> {
        if let serde_yaml::Value::String(s) = value {
            if let Some(max_length) = self.max_length {
                if (s.len() as u64) > max_length {
                    return Err(YamlSchemaError::GenericError(format!(
                        "String length is greater than max_length: {}",
                        max_length
                    )));
                }
            }
            if let Some(min_length) = self.min_length {
                if (s.len() as u64) < min_length {
                    return Err(YamlSchemaError::GenericError(format!(
                        "String length is less than min_length: {}",
                        min_length
                    )));
                }
            }
            if let Some(pattern) = &self.pattern {
                let re = regex::Regex::new(pattern).map_err(|e| {
                    YamlSchemaError::GenericError(format!("Invalid regex pattern: {}", e))
                })?;
                if !re.is_match(s) {
                    return Err(YamlSchemaError::GenericError(format!(
                        "String does not match pattern: {}",
                        pattern
                    )));
                }
            }
            Ok(())
        } else {
            Err(YamlSchemaError::GenericError(format!(
                "Expected string, got {:?}",
                value
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string_literal() {
        let inputs = [
            r#"
            type: "string"
            "#,
            r#"
            type: "string"
            maxLength: 10
            "#,
            r#"
            type: "string"
            minLength: 1
            "#,
            r#"
            type: "string"
            pattern: "^[a-z]+$"
            "#,
        ];
        let expecteds = [
            Literal::String(YamlString {
                max_length: None,
                min_length: None,
                pattern: None,
            }),
            Literal::String(YamlString {
                max_length: Some(10),
                min_length: None,
                pattern: None,
            }),
            Literal::String(YamlString {
                max_length: None,
                min_length: Some(1),
                pattern: None,
            }),
            Literal::String(YamlString {
                max_length: None,
                min_length: None,
                pattern: Some("^[a-z]+$".to_string()),
            }),
        ];
        for (expected, input) in expecteds.iter().zip(inputs.iter()) {
            println!("input: {}", input);
            println!("expected: {:?}", expected);
            let actual = serde_yaml::from_str(input).unwrap();
            assert_eq!(*expected, actual);
        }
    }

    #[test]
    fn test_validate_string_literal() {
        let yaml_string = YamlString {
            max_length: Some(10),
            min_length: Some(1),
            pattern: Some("^[a-z]+$".to_string()),
        };
        let value = serde_yaml::Value::String("hello".to_string());
        assert!(yaml_string.validate(&value).is_ok());
        let value = serde_yaml::Value::String("hello world".to_string());
        if let YamlSchemaError::GenericError(s) = yaml_string.validate(&value).unwrap_err() {
            assert_eq!(s, "String length is greater than max_length: 10");
        }
        let value = serde_yaml::Value::String("".to_string());
        if let YamlSchemaError::GenericError(s) = yaml_string.validate(&value).unwrap_err() {
            assert_eq!(s, "String length is less than min_length: 1");
        }
        let value = serde_yaml::Value::String("123".to_string());
        if let YamlSchemaError::GenericError(s) = yaml_string.validate(&value).unwrap_err() {
            assert_eq!(s, "String does not match pattern: ^[a-z]+$");
        }
    }
}
