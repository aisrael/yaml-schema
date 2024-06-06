use log::debug;

use crate::error::YamlSchemaError;
use crate::literals::{Literal, YamlString};
use crate::Validator;
use crate::YamlSchema;

pub struct Engine {
    pub schema: YamlSchema,
}

impl Engine {
    pub fn new(schema: YamlSchema) -> Engine {
        Engine { schema }
    }

    pub fn evaluate(&self, yaml: &serde_yaml::Value) -> Result<(), YamlSchemaError> {
        debug!("Engine is running");

        validate(&self.schema, yaml)
    }
}

fn validate(schema: &YamlSchema, value: &serde_yaml::Value) -> Result<(), YamlSchemaError> {
    match schema {
        YamlSchema::Literal(literal) => validate_literal(literal, value),
        _ => unimplemented!(),
    }
}

fn validate_literal(literal: &Literal, value: &serde_yaml::Value) -> Result<(), YamlSchemaError> {
    match literal {
        Literal::String(yaml_string) => yaml_string.validate(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine() {
        let literal = Literal::String(YamlString::with_min_length(1));
        let schema = YamlSchema::Literal(literal);
        let engine = Engine::new(schema);
        let yaml: serde_yaml::Value = serde_yaml::from_str(r#""hello""#).unwrap();
        let res = engine.evaluate(&yaml);
        assert!(res.is_ok());

        let invalid_yaml = serde_yaml::from_str(r#""""#).unwrap();
        assert!(engine.evaluate(&invalid_yaml).is_err());
    }
}
