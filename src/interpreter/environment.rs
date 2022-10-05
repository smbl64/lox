use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::RuntimeError;
use crate::token::{Object, Token};

#[derive(Debug)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        Self::with_enclosing(None)
    }

    pub fn with_enclosing(enclosing: Option<Rc<RefCell<Environment>>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing,
        }
    }

    pub fn define(&mut self, name: &str, value: Object) {
        self.values.insert(name.to_owned(), value);
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<(), RuntimeError> {
        if !self.values.contains_key(&name.lexeme) {
            // Ask one level above if possible
            if let Some(ref e) = self.enclosing {
                return e.borrow_mut().assign(name, value);
            }

            return Err(RuntimeError::UndefinedVariable {
                name: name.clone(),
                msg: format!("Undefined variable '{}'", name.lexeme),
            });
        }

        self.values.insert(name.lexeme.clone(), value);
        Ok(())
    }

    pub fn get(&self, name: &Token) -> Result<Object, RuntimeError> {
        let value = self.values.get(&name.lexeme).map(|lit| lit.to_owned());
        // Ask one level above if possible
        if value.is_none() && self.enclosing.is_some() {
            let rc = self.enclosing.as_ref().unwrap();
            return rc.borrow_mut().get(name);
        }

        value.ok_or_else(move || RuntimeError::UndefinedVariable {
            name: name.clone(),
            msg: format!("Undefined variable '{}'", name.lexeme),
        })
    }
}
