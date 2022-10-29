use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::RuntimeInterrupt;
use crate::object::Object;
use crate::token::Token;

#[derive(Debug, Default)]
pub struct Environment {
    pub enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_enclosing(self, enclosing: Rc<RefCell<Environment>>) -> Self {
        Self { enclosing: Some(enclosing), ..Default::default() }
    }

    pub fn as_shared(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }

    pub fn define(&mut self, name: &str, value: Object) {
        self.values.insert(name.to_owned(), value);
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<(), RuntimeInterrupt> {
        if !self.values.contains_key(&name.lexeme) {
            // Ask one level above if possible
            if let Some(ref e) = self.enclosing {
                return e.borrow_mut().assign(name, value);
            }

            return Err(RuntimeInterrupt::error(
                name.line,
                format!("Undefined variable '{}'", name.lexeme),
            ));
        }

        self.values.insert(name.lexeme.clone(), value);
        Ok(())
    }

    pub fn assign_at(
        &mut self,
        distance: usize,
        name: &Token,
        value: Object,
    ) -> Result<(), RuntimeInterrupt> {
        if distance == 0 {
            return self.assign(name, value);
        }

        match self.ancestor(distance) {
            None => Err(RuntimeInterrupt::error(
                name.line,
                format!("No enclosing environment at {distance} for '{}'", name.lexeme),
            )),
            Some(ancestor) => ancestor.borrow_mut().assign(name, value),
        }
    }

    pub fn get(&self, name: &Token) -> Result<Object, RuntimeInterrupt> {
        let value = self.values.get(&name.lexeme).map(|lit| lit.to_owned());
        // Ask one level above if possible
        if value.is_none() && self.enclosing.is_some() {
            let rc = self.enclosing.as_ref().unwrap();
            return rc.borrow_mut().get(name);
        }

        value.ok_or_else(|| {
            RuntimeInterrupt::error(name.line, format!("Undefined variable '{}'", name.lexeme))
        })
    }

    pub fn get_at(&self, distance: usize, name: &Token) -> Result<Object, RuntimeInterrupt> {
        if distance == 0 {
            return self.get(name);
        }

        match self.ancestor(distance) {
            None => Err(RuntimeInterrupt::error(
                name.line,
                format!("No enclosing environment at {distance} for '{}'", name.lexeme),
            )),
            Some(ancestor) => ancestor.borrow().get(name),
        }
    }

    fn ancestor(&self, distance: usize) -> Option<Rc<RefCell<Environment>>> {
        let parent = self.enclosing.clone()?;
        let mut env = parent;

        for _ in 1..distance {
            let parent = env.borrow().enclosing.clone()?;
            env = parent.clone();
        }
        Some(env)
    }
}
