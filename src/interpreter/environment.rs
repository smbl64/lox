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
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(enclosing),
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

    pub fn assign_at(
        &mut self,
        distance: usize,
        name: &Token,
        value: Object,
    ) -> Result<(), RuntimeError> {
        if distance == 0 {
            return self.assign(name, value);
        }

        match self.ancestor(distance) {
            None => Err(RuntimeError::UndefinedVariable {
                name: name.clone(),
                msg: format!(
                    "No enclosing environment at {} for '{}'",
                    distance, name.lexeme
                ),
            }),
            Some(ancestor) => ancestor.borrow_mut().assign(name, value),
        }
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

    pub fn get_at(&self, distance: usize, name: &Token) -> Result<Object, RuntimeError> {
        if distance == 0 {
            return self.get(name);
        }

        match self.ancestor(distance) {
            None => Err(RuntimeError::UndefinedVariable {
                name: name.clone(),
                msg: format!(
                    "No enclosing environment at {} for '{}'",
                    distance, name.lexeme
                ),
            }),
            Some(ancestor) => ancestor.borrow().get(name),
        }
    }

    fn ancestor(&self, distance: usize) -> Option<Rc<RefCell<Environment>>> {
        let parent = self.enclosing.clone()?;
        let mut env = parent.clone();

        for _ in 1..distance {
            let parent = env.borrow().enclosing.clone()?;
            env = parent.clone();
        }
        Some(env)
    }
}
