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
    fn validate(&self, context: &Context, value: &saphyr::MarkedYaml) -> Result<()> {
        let data = &value.data;
        debug!(
            "Validating value: {:?} against const: {:?}",
            &data, self.r#const
        );
        let expected_value = &self.r#const;
        match expected_value {
            ConstValue::Boolean(b) => {
                if data.as_bool() != Some(*b) {
                    let error = format!(
                        "Const validation failed, expected: {:?}, got: {:?}",
                        b, data
                    );
                    context.add_error(error);
                }
            }
            ConstValue::Null => {
                if !data.is_null() {
                    let error = format!("Const validation failed, expected: null, got: {:?}", data);
                    context.add_error(error);
                }
            }
            ConstValue::Number(n) => match n {
                Number::Integer(i) => {
                    if data.is_integer() {
                        if data.as_i64() != Some(*i) {
                            let error = format!(
                                "Const validation failed, expected: {:?}, got: {:?}",
                                i, data
                            );
                            context.add_error(error);
                        }
                    } else {
                        let error = format!(
                            "Const validation failed, expected: {:?}, got: {:?}",
                            i, data
                        );
                        context.add_error(error);
                    }
                }
                Number::Float(f) => {
                    if data.is_real() {
                        if data.as_f64() != Some(*f) {
                            let error = format!(
                                "Const validation failed, expected: {:?}, got: {:?}",
                                f, data
                            );
                            context.add_error(error);
                        }
                    } else {
                        let error = format!(
                            "Const validation failed, expected: {:?}, got: {:?}",
                            f, data
                        );
                        context.add_error(error);
                    }
                }
            },
            ConstValue::String(s) => {
                if data.as_str() != Some(s) {
                    let error = format!(
                        "Const validation failed, expected: {:?}, got: {:?}",
                        s, data
                    );
                    context.add_error(error);
                }
            }
        }
        Ok(())
    }
}
