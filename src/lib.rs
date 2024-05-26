use serde::{Deserialize, Serialize};

mod error;
mod literals;

pub use error::YamlSchemaError;
pub use literals::{Literal, YamlString};

pub trait Validator {
    fn validate(&self, value: &serde_yaml::Value) -> Result<(), YamlSchemaError>;
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum YamlSchema {
    AnyOf {
        #[serde(rename = "anyOf")]
        any_of: Vec<Literal>,
    },
    AllOf {
        #[serde(rename = "allOf")]
        all_of: Vec<Literal>,
    },
    Enum {
        #[serde(rename = "enum")]
        values: Vec<serde_yaml::Value>,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum EnumValue {
    String(String),
    Integer(i64),
    Float(f64),
    Literal(Literal),
}

#[cfg(test)]
mod tests {

    use std::vec;

    use super::*;

    #[test]
    fn test_any_of() {
        let inputs = [r#"
            anyOf:
                - type: "string"
                  minLength: 1
            "#];
        let expecteds = [YamlSchema::AnyOf {
            any_of: vec![Literal::String(YamlString {
                max_length: None,
                min_length: Some(1),
                pattern: None,
            })],
        }];
        for (expected, input) in expecteds.iter().zip(inputs.iter()) {
            let actual = serde_yaml::from_str(input).unwrap();
            assert_eq!(*expected, actual);
        }
    }

    #[test]
    fn test_all_of() {
        let inputs = [r#"
            allOf:
                - type: "string"
                  minLength: 1
            "#];
        let expecteds = [YamlSchema::AllOf {
            all_of: vec![Literal::String(YamlString {
                max_length: None,
                min_length: Some(1),
                pattern: None,
            })],
        }];
        for (expected, input) in expecteds.iter().zip(inputs.iter()) {
            let actual = serde_yaml::from_str(input).unwrap();
            assert_eq!(*expected, actual);
        }
    }

    #[test]
    fn test_enum() {
        let inputs = [r#"
            enum:
                - null
            "#];
        let expecteds = [YamlSchema::Enum {
            values: vec![serde_yaml::Value::Null],
        }];
        for (expected, input) in expecteds.iter().zip(inputs.iter()) {
            let actual = serde_yaml::from_str(input).unwrap();
            assert_eq!(*expected, actual);
        }
    }
}
