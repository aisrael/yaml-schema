use log::debug;

use crate::error::YamlSchemaError;
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
        self.schema.validate(yaml)
    }
}

#[cfg(test)]
mod tests {

    // fn test_engine() {
    //     let literal = Literal::String(YamlString::with_min_length(1));
    //     let schema = YamlSchema::new();
    //     let engine = Engine::new(schema);
    //     let yaml: serde_yaml::Value = serde_yaml::from_str(r#""hello""#).unwrap();
    //     let res = engine.evaluate(&yaml);
    //     assert!(res.is_ok());

    //     let invalid_yaml = serde_yaml::from_str(r#""""#).unwrap();
    //     assert!(engine.evaluate(&invalid_yaml).is_err());
    // }
}
