use log::debug;

use crate::format_serde_yaml_value;
use crate::format_vec;
use crate::ConstValue;
use crate::Context;
use crate::Result;
use crate::Validator;

/// An enum schema represents a set of constant values
#[derive(Debug, Default, PartialEq)]
pub struct EnumSchema {
    pub r#enum: Vec<ConstValue>,
}

impl std::fmt::Display for EnumSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Enum {{ enum: {} }}", format_vec(&self.r#enum))
    }
}

impl Validator for EnumSchema {
    fn validate(&self, context: &Context, value: &serde_yaml::Value) -> Result<()> {
        debug!("[EnumSchema] self: {}", self);
        debug!("[EnumSchema] Validating value: {:?}", value);
        let const_value = ConstValue::from_serde_yaml_value(&value);
        debug!("[EnumSchema] const_value: {}", const_value);
        if !self.r#enum.contains(&const_value) {
            let value_str = format_serde_yaml_value(value);
            let enum_values = self
                .r#enum
                .iter()
                .map(|v| format!("{}", v))
                .collect::<Vec<String>>()
                .join(", ");
            let error = format!("Value {} is not in the enum: [{}]", value_str, enum_values);
            debug!("[EnumSchema] error: {}", error);
            context.add_error(error);
        }
        Ok(())
    }
}
