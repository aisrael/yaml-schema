use std::cell::RefCell;
use std::rc::Rc;

use crate::validation::ValidationError;
use crate::YamlSchema;

/// The validation context
#[derive(Debug, Default)]
pub struct Context {
    pub current_schema: Option<Rc<YamlSchema>>,
    pub current_path: Vec<String>,
    pub stream_started: bool,
    pub stream_ended: bool,
    pub errors: Rc<RefCell<Vec<ValidationError>>>,
    pub fail_fast: bool,
}

impl Context {
    /// Returns true if there are any errors in the context
    pub fn has_errors(&self) -> bool {
        !self.errors.borrow().is_empty()
    }

    /// Returns the current path as a string separated by "."
    pub fn path(&self) -> String {
        self.current_path.join(".")
    }

    pub fn new(fail_fast: bool) -> Context {
        Context {
            fail_fast,
            ..Default::default()
        }
    }

    pub fn push_error(&self, error: ValidationError) {
        self.errors.borrow_mut().push(error);
    }

    pub fn add_doc_error<V: Into<String>>(&self, error: V) {
        let path = self.path();
        self.push_error(ValidationError {
            path,
            line_col: None,
            error: error.into(),
        });
    }

    pub fn add_error<V: Into<String>>(&self, marked_yaml: &saphyr::MarkedYaml, error: V) {
        let path = self.path();
        self.push_error(ValidationError {
            path,
            line_col: Some(marked_yaml.into()),
            error: error.into(),
        });
    }

    /// Append a path to the current path
    pub fn append_path<V: Into<String>>(&self, path: V) -> Context {
        let mut new_path = self.current_path.clone();
        new_path.push(path.into());
        Context {
            current_schema: self.current_schema.clone(),
            current_path: new_path,
            errors: self.errors.clone(),
            fail_fast: self.fail_fast,
            stream_ended: self.stream_ended,
            stream_started: self.stream_started,
        }
    }
}
