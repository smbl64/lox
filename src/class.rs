use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use crate::object::Object;
use crate::prelude::RuntimeError;
use crate::token::Token;

#[derive(Debug, Clone)]
pub struct Class {
    name: String,
}

impl Class {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().to_owned(),
        }
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Class {
    pub fn construct(class: Rc<RefCell<Class>>) -> Instance {
        Instance::new(class)
    }
}

#[derive(Debug, Clone)]
pub struct Instance {
    class: Rc<RefCell<Class>>,
    fields: HashMap<String, Object>,
}

impl Instance {
    pub fn new(class: Rc<RefCell<Class>>) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, field: &Token) -> Result<Object, RuntimeError> {
        if let Some(object) = self.fields.get(&field.lexeme) {
            Ok(object.clone())
        } else {
            Err(RuntimeError::UndefinedVariable {
                name: field.clone(),
                msg: format!("Undefined property '{}'", field.lexeme),
            })
        }
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class.borrow())
    }
}
