use log::debug;

use crate::error::YamlSchemaError;
use crate::literals::Literal;
use crate::not_yet_implemented;
use crate::Validator;
use crate::YamlSchema;

pub struct Engine<'a> {
    pub schema: &'a YamlSchema,
}

impl<'a> Engine<'a> {
    pub fn new(schema: &'a YamlSchema) -> Engine<'a> {
        Engine { schema }
    }

    pub fn evaluate(&self, yaml: &serde_yaml::Value) -> Result<(), YamlSchemaError> {
        debug!("Engine is running");

        validate(&self.schema, yaml)
    }
}

fn validate(schema: &YamlSchema, value: &serde_yaml::Value) -> Result<(), YamlSchemaError> {
    Ok(())
}

fn validate_literal(literal: &Literal, value: &serde_yaml::Value) -> Result<(), YamlSchemaError> {
    match literal {
        Literal::String(yaml_string) => yaml_string.validate(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::literals::YamlString;

    #[test]
    fn test_engine() {
        let literal = Literal::String(YamlString::with_min_length(1));
        let schema = YamlSchema::new();
        let engine = Engine::new(schema);
        let yaml: serde_yaml::Value = serde_yaml::from_str(r#""hello""#).unwrap();
        let res = engine.evaluate(&yaml);
        assert!(res.is_ok());

        let invalid_yaml = serde_yaml::from_str(r#""""#).unwrap();
        assert!(engine.evaluate(&invalid_yaml).is_err());
    }
}
