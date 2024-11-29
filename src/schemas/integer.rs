use eyre::Result;
use std::fmt;

use crate::validation::Context;
use crate::validation::Validator;
use crate::Number;
use crate::YamlSchemaError;

/// A number schema
#[derive(Debug, Default, PartialEq)]
pub struct IntegerSchema {
    pub minimum: Option<Number>,
    pub maximum: Option<Number>,
    pub exclusive_minimum: Option<Number>,
    pub exclusive_maximum: Option<Number>,
    pub multiple_of: Option<Number>,
}

impl fmt::Display for IntegerSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Number {:?}", self)
    }
}

impl Validator for IntegerSchema {
    fn validate(&self, context: &Context, value: &serde_yaml::Value) -> Result<()> {
        match value.as_i64() {
            Some(i) => self.validate_number_i64(context, i),
            None => {
                context.add_error(format!("Expected an integer, but got: {:?}", value));
            }
        }
        if !context.errors.borrow().is_empty() {
            fail_fast!(context)
        }
        Ok(())
    }
}

impl IntegerSchema {
    fn validate_number_i64(&self, context: &Context, i: i64) {
        if let Some(minimum) = &self.minimum {
            match minimum {
                Number::Integer(min) => {
                    if i < *min {
                        context.add_error("Number is too small!".to_string());
                    }
                }
                Number::Float(min) => {
                    if (i as f64) < *min {
                        context.add_error("Number is too small!".to_string());
                    }
                }
            }
        }
        if let Some(maximum) = &self.maximum {
            match maximum {
                Number::Integer(max) => {
                    if i > *max {
                        context.add_error("Number is too big!".to_string());
                    }
                }
                Number::Float(max) => {
                    if (i as f64) > *max {
                        context.add_error("Number is too big!".to_string());
                    }
                }
            }
        }
        if let Some(multiple_of) = &self.multiple_of {
            match multiple_of {
                Number::Integer(multiple) => {
                    if i % *multiple != 0 {
                        context.add_error(format!("Number is not a multiple of {}!", multiple));
                    }
                }
                Number::Float(multiple) => {
                    if (i as f64) % *multiple != 0.0 {
                        context.add_error(format!("Number is not a multiple of {}!", multiple));
                    }
                }
            }
        }
    }
}
