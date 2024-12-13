use crate::Number;
use log::debug;

use crate::ConstValue;
use crate::Context;
use crate::Result;

use super::Validator;

/// A const schema represents a constant value
#[derive(Debug, PartialEq)]
pub struct ConstSchema {
    pub r#const: ConstValue,
}

impl std::fmt::Display for ConstSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Const {:?}", self.r#const)
    }
}

impl Validator for ConstSchema {
    fn validate(&self, context: &Context, value: &serde_yaml::Value) -> Result<()> {
        debug!(
            "Validating value: {:?} against const: {:?}",
            value, self.r#const
        );
        let expected_value = &self.r#const;
        match expected_value {
            ConstValue::Boolean(b) => {
                if value.as_bool() != Some(*b) {
                    let error = format!(
                        "Const validation failed, expected: {:?}, got: {:?}",
                        b, value
                    );
                    context.add_error(error);
                }
            }
            ConstValue::Null => {
                if !value.is_null() {
                    let error =
                        format!("Const validation failed, expected: null, got: {:?}", value);
                    context.add_error(error);
                }
            }
            ConstValue::Number(n) => match n {
                Number::Integer(i) => {
                    if value.is_i64() {
                        if value.as_i64() != Some(*i) {
                            let error = format!(
                                "Const validation failed, expected: {:?}, got: {:?}",
                                i, value
                            );
                            context.add_error(error);
                        }
                    } else {
                        let error = format!(
                            "Const validation failed, expected: {:?}, got: {:?}",
                            i, value
                        );
                        context.add_error(error);
                    }
                }
                Number::Float(f) => {
                    if value.is_f64() {
                        if value.as_f64() != Some(*f) {
                            let error = format!(
                                "Const validation failed, expected: {:?}, got: {:?}",
                                f, value
                            );
                            context.add_error(error);
                        }
                    } else {
                        let error = format!(
                            "Const validation failed, expected: {:?}, got: {:?}",
                            f, value
                        );
                        context.add_error(error);
                    }
                }
            },
            ConstValue::String(s) => {
                if value.as_str() != Some(s) {
                    let error = format!(
                        "Const validation failed, expected: {:?}, got: {:?}",
                        s, value
                    );
                    context.add_error(error);
                }
            }
        }
        Ok(())
    }
}
