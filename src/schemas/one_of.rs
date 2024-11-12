use serde::{Deserialize, Serialize};
/// The `oneOf` schema is a schema that matches if any of the schemas in the `oneOf` array match.
/// The schemas are tried in order, and the first match is used. If no match is found, an error is added
/// to the context.
use std::fmt;

use crate::{format_vec, YamlSchema};

/// The `oneOf` schema is a schema that matches if any of the schemas in the `oneOf` array match.
/// The schemas are tried in order, and the first match is used. If no match is found, an error is added
/// to the context.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OneOfSchema {
    pub one_of: Vec<YamlSchema>,
}

impl fmt::Display for OneOfSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "oneOf:{}", format_vec(&self.one_of))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{Context, Validator};
    use crate::{TypeValue, TypedSchema, YamlSchema, YamlSchemaNumber};

    #[test]
    fn test_one_of_schema() {
        let schemas = vec![
            YamlSchema::TypedSchema(Box::new(TypedSchema {
                r#type: TypeValue::number(),
                multiple_of: Some(YamlSchemaNumber::Integer(5)),
                ..Default::default()
            })),
            YamlSchema::TypedSchema(Box::new(TypedSchema {
                r#type: TypeValue::number(),
                multiple_of: Some(YamlSchemaNumber::Integer(3)),
                ..Default::default()
            })),
        ];

        let schema = OneOfSchema { one_of: schemas };
        println!("{}", schema);
        let root_schema = YamlSchema::OneOf(schema);
        let context = Context::new(&root_schema, false);
        assert!(root_schema
            .validate(
                &context,
                &serde_yaml::Value::Number(serde_yaml::Number::from(5.0))
            )
            .is_ok());
    }
}
