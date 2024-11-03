use crate::error::YamlSchemaError;
use crate::YamlSchema;

pub struct Context<'a> {
    pub current_schema: &'a YamlSchema,
    pub current_path: Vec<String>,
    pub errors: Vec<YamlSchemaError>,
}

impl<'a> Context<'a> {
    pub fn new(root_schema: &'a YamlSchema) -> Context<'a> {
        Context {
            current_schema: root_schema,
            current_path: vec![],
            errors: vec![],
        }
    }

    pub fn add_error(&mut self, error: YamlSchemaError) {
        self.errors.push(error);
    }

    pub fn append_path(&mut self, path: &str) {
        self.current_path.push(path.to_string());
    }
}
