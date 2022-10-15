use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use crate::object::Object;
use crate::prelude::{LoxFunction, RuntimeError};
use crate::token::Token;

#[derive(Debug, Clone)]
pub struct Class {
    name: String,
    methods: HashMap<String, Rc<LoxFunction>>,
}

impl Class {
    pub fn new(name: impl AsRef<str>, methods: HashMap<String, Rc<LoxFunction>>) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            methods,
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

    pub fn find_method(&self, name: &str) -> Option<Rc<LoxFunction>> {
        self.methods.get(name).map(|f| f.clone())
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

    pub fn get(&self, field: &Token, instance: &Object) -> Result<Object, RuntimeError> {
        if let Some(object) = self.fields.get(&field.lexeme) {
            Ok(object.clone())
        } else if let Some(function) = self.class.borrow().find_method(&field.lexeme) {
            let function = function.bind(instance.clone());

            Ok(Object::Callable(function))
        } else {
            Err(RuntimeError::UndefinedVariable {
                name: field.clone(),
                msg: format!("Undefined property '{}'", field.lexeme),
            })
        }
    }

    pub fn set(&mut self, field: &Token, value: Object) {
        self.fields.insert(field.lexeme.clone(), value);
    }

    pub fn unique_id(&self) -> usize {
        std::ptr::addr_of!(*self) as usize
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class.borrow())
    }
}
