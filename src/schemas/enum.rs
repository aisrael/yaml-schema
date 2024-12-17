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
    fn validate(&self, context: &Context, value: &saphyr::Yaml) -> Result<()> {
        debug!("[EnumSchema] self: {}", self);
        debug!("[EnumSchema] Validating value: {:?}", value);
        let const_value = ConstValue::from_saphyr_yaml(value);
        debug!("[EnumSchema] const_value: {}", const_value);
        for value in &self.r#enum {
            debug!("[EnumSchema] value: {}", value);
            if value.eq(&const_value) {
                return Ok(());
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enum_schema() {
        let schema = EnumSchema {
            r#enum: vec![ConstValue::String("NW".to_string())],
        };
        let value = saphyr::Yaml::String("NW".to_string());
        let context = Context::default();
        let result = schema.validate(&context, &value);
        assert!(result.is_ok());
    }
}
