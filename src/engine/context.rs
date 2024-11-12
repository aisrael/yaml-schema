use crate::YamlSchema;
use std::cell::RefCell;
use std::rc::Rc;

use crate::validation::ValidationError;

/// The validation context
pub struct Context<'a> {
    pub current_schema: &'a YamlSchema,
    pub current_path: Vec<String>,
    pub errors: Rc<RefCell<Vec<ValidationError>>>,
    pub fail_fast: bool,
}

impl<'a> Context<'a> {
    /// Returns true if there are any errors in the context
    pub fn has_errors(&self) -> bool {
        !self.errors.borrow().is_empty()
    }

    pub fn path(&self) -> String {
        self.current_path.join(".")
    }

    pub fn new(root_schema: &'a YamlSchema, fail_fast: bool) -> Context<'a> {
        Context {
            current_schema: root_schema,
            current_path: vec![],
            errors: Rc::new(RefCell::new(Vec::new())),
            fail_fast,
        }
    }

    pub fn add_error<V: Into<String>>(&self, error: V) {
        let path = self.path();
        self.errors.borrow_mut().push(ValidationError {
            path,
            error: error.into(),
        });
    }

    pub fn append_path<V: Into<String>>(&self, path: V) -> Context<'a> {
        let mut new_path = self.current_path.clone();
        new_path.push(path.into());
        Context {
            current_schema: self.current_schema,
            current_path: new_path,
            errors: self.errors.clone(),
            fail_fast: self.fail_fast,
        }
    }
}
