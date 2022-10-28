use std::fmt::Display;
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;
use crate::object::Object;
use crate::prelude::Callable;

#[derive(Debug)]
struct Clock;

impl Callable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _interpret: &mut Interpreter,
        _arguments: Vec<Object>,
    ) -> Result<Object, RuntimeError> {
        let start = SystemTime::now();
        let since_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backward");

        Ok(Object::Number(since_epoch.as_millis() as f64 / 1000.0))
    }
}

impl Display for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}

pub fn clock() -> Rc<dyn Callable> {
    Rc::new(Clock)
}
