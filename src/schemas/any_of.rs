use crate::YamlSchema;

/// AnyOf
#[derive(Debug)]
pub struct AnyOfSchema {
    schemas: Vec<YamlSchema>,
}

impl AnyOfSchema {}

impl std::fmt::Display for AnyOfSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, schema) in self.schemas.iter().enumerate() {
            write!(f, "{}", schema)?;
            if i < self.schemas.len() - 1 {
                write!(f, " | ")?;
            }
        }
        Ok(())
    }
}
