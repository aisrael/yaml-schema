use std::fmt;

/// A string schema
#[derive(Debug, PartialEq)]
pub struct StringSchema {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
}

impl fmt::Display for StringSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "String {:?}", self)
    }
}
