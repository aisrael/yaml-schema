use crate::validation::Context;
use crate::validation::Validator;
use crate::Number;
use crate::Result;

/// A number schema
#[derive(Debug, Default, PartialEq)]
pub struct NumberSchema {
    pub minimum: Option<Number>,
    pub maximum: Option<Number>,
    pub exclusive_minimum: Option<Number>,
    pub exclusive_maximum: Option<Number>,
    pub multiple_of: Option<Number>,
}

impl std::fmt::Display for NumberSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Number {:?}", self)
    }
}

impl Validator for NumberSchema {
    fn validate(&self, context: &Context, value: &serde_yaml::Value) -> Result<()> {
        if value.is_i64() {
            match value.as_i64() {
                Some(i) => self.validate_number_i64(context, i),
                None => {
                    context.add_error(format!("Expected an integer, but got: {:?}", value));
                }
            }
        } else if value.is_f64() {
            match value.as_f64() {
                Some(f) => self.validate_number_f64(context, f),
                None => {
                    context.add_error(format!("Expected a float, but got: {:?}", value));
                }
            }
        } else {
            context.add_error(format!("Expected a number, but got: {:?}", value));
        }
        if !context.errors.borrow().is_empty() {
            fail_fast!(context)
        }
        Ok(())
    }
}

impl NumberSchema {
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
    fn validate_number_f64(&self, context: &Context, f: f64) {
        if let Some(minimum) = &self.minimum {
            match minimum {
                Number::Integer(min) => {
                    if f < *min as f64 {
                        context.add_error("Number is too small!".to_string());
                    }
                }
                Number::Float(min) => {
                    if f < *min {
                        context.add_error("Number is too small!".to_string());
                    }
                }
            }
        }
        if let Some(maximum) = &self.maximum {
            match maximum {
                Number::Integer(max) => {
                    if f > *max as f64 {
                        context.add_error("Number is too big!".to_string());
                    }
                }
                Number::Float(max) => {
                    if f > *max {
                        context.add_error("Number is too big!".to_string());
                    }
                }
            }
        }
    }
}
