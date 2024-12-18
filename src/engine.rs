use std::cell::RefCell;
use std::rc::Rc;

use crate::validation::Context;
use crate::Error;
use crate::Result;
use crate::RootSchema;
use crate::YamlSchema;

#[derive(Debug)]
pub struct Engine<'a> {
    pub root_schema: &'a RootSchema,
    pub context: Rc<RefCell<Context>>,
}

impl<'a> Engine<'a> {
    pub fn new(root_schema: &'a RootSchema, context: Context) -> Self {
        Engine {
            root_schema,
            context: Rc::new(RefCell::new(context)),
        }
    }

    pub fn evaluate<'b: 'a>(
        root_schema: &'a RootSchema,
        value: &str,
        fail_fast: bool,
    ) -> Result<Context> {
        let context = Context {
            current_schema: Some(root_schema.schema.clone()),
            fail_fast,
            ..Default::default()
        };
        let engine = Engine::new(root_schema, context);
        let docs = saphyr::MarkedYaml::load_from_str(value).map_err(Error::YamlParsingError)?;
        if docs.is_empty() {
            match root_schema.schema.as_ref() {
                YamlSchema::Empty => (),
                YamlSchema::BooleanLiteral(false) => {
                    engine
                        .context
                        .borrow_mut()
                        .add_error("Empty YAML document is not allowed");
                }
                YamlSchema::BooleanLiteral(true) => (),
                _ => engine
                    .context
                    .borrow_mut()
                    .add_error("Empty YAML document is not allowed"),
            }
        } else {
            let yaml = docs.first().unwrap();
            engine
                .root_schema
                .validate(&engine.context.borrow(), yaml)?;
        }
        Ok(engine.context.take())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_empty_schema() {
        let root_schema = RootSchema::new(YamlSchema::Empty);
        let context = Engine::evaluate(&root_schema, "", false).unwrap();
        assert!(!context.has_errors());
    }

    #[test]
    fn test_engine_boolean_literal_true() {
        let root_schema = RootSchema::new(YamlSchema::BooleanLiteral(true));
        let context = Engine::evaluate(&root_schema, "", false).unwrap();
        assert!(!context.has_errors());
    }

    #[test]
    fn test_engine_boolean_literal_false() {
        let root_schema = RootSchema::new(YamlSchema::BooleanLiteral(false));
        let context = Engine::evaluate(&root_schema, "", false).unwrap();
        assert!(context.has_errors());
    }
}
