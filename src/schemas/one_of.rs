/// The `oneOf` schema is a schema that matches if any of the schemas in the `oneOf` array match.
/// The schemas are tried in order, and the first match is used. If no match is found, an error is added
/// to the context.
use std::fmt;

use crate::{format_vec, YamlSchema};

/// The `oneOf` schema is a schema that matches if any of the schemas in the `oneOf` array match.
/// The schemas are tried in order, and the first match is used. If no match is found, an error is added
/// to the context.
#[derive(Debug, Default, PartialEq)]
pub struct OneOfSchema {
    pub one_of: Vec<YamlSchema>,
}

impl fmt::Display for OneOfSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "oneOf:{}", format_vec(&self.one_of))
    }
}

impl From<&crate::deser::OneOfSchema> for OneOfSchema {
    fn from(source: &crate::deser::OneOfSchema) -> Self {
        let one_of: Vec<YamlSchema> = source.one_of.iter().map(|s| s.into()).collect();
        OneOfSchema { one_of }
    }
}
