use crate::prelude::*;
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Object {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Callable(Rc<dyn Callable>),
    Class(Rc<RefCell<Class>>),
    Instance(Instance),
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Boolean(left), Self::Boolean(right)) => left == right,
            (Self::Number(left), Self::Number(right)) => left == right,
            (Self::String(left), Self::String(right)) => left == right,
            (Self::Callable(_), Self::Callable(_)) => false,
            _ => false,
        }
    }
}

impl Eq for Object {}

impl Object {
    pub fn number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }
    pub fn boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }
    pub fn string(&self) -> Option<String> {
        match self {
            Self::String(s) => Some(s.clone()),
            _ => None,
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Boolean(b) => write!(f, "{}", b),
            Self::Number(n) => {
                write!(f, "{}", n)
            }
            Self::String(s) => write!(f, "{}", s),
            Self::Null => write!(f, "nil"),
            Self::Callable(c) => write!(f, "{}", c),
            Self::Class(c) => write!(f, "{}", c.borrow()),
            Self::Instance(i) => write!(f, "{}", i),
        }
    }
}
