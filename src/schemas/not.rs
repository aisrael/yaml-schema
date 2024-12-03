/// The `not` keyword declares that an instance validates if it doesn't validate against the given subschema.
use crate::YamlSchema;

/// The `not` keyword declares that an instance validates if it doesn't validate against the given subschema.
#[derive(Debug, Default, PartialEq)]
pub struct NotSchema {
    pub not: Box<YamlSchema>,
}

impl std::fmt::Display for NotSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not: {}", self.not)
    }
}
