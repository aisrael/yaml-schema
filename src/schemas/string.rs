use regex::Regex;

use crate::Result;
use crate::{validation::strings::validate_string, Context, Validator};

/// A string schema
#[derive(Debug, Default)]
pub struct StringSchema {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<Regex>,
}

impl PartialEq for StringSchema {
    fn eq(&self, other: &Self) -> bool {
        self.min_length == other.min_length
            && self.max_length == other.max_length
            && are_patterns_equal(&self.pattern, &other.pattern)
    }
}

/// 'Naive' check to see if two regexes are equal, by comparing their string representations
/// We do it this way because we can't `impl PartialEq for Regex` and don't want to have to
/// alias or wrap the `regex::Regex` type
fn are_patterns_equal(a: &Option<Regex>, b: &Option<Regex>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => a.as_str() == b.as_str(),
        (None, None) => true,
        _ => false,
    }
}

impl std::fmt::Display for StringSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "String {:?}", self)
    }
}

impl Validator for StringSchema {
    fn validate(&self, context: &Context, value: &serde_yaml::Value) -> Result<()> {
        match validate_string(
            self.min_length,
            self.max_length,
            self.pattern.as_ref(),
            value,
        ) {
            Ok(errors) => {
                if !errors.is_empty() {
                    for error in errors {
                        context.add_error(error);
                    }
                }
                Ok(())
            }
            Err(e) => generic_error!("{}", e),
        }
    }
}
