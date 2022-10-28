mod expr;
mod stmt;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::prelude::*;
use crate::SharedErrorReporter;

type InterpreterResult = Result<Object, RuntimeError>;

pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
    locals: HashMap<UniqueId, usize>, // unique id -> depth
    error_reporter: Option<SharedErrorReporter>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Environment::new().as_shared();
        let environment = globals.clone();

        globals.borrow_mut().define("clock", Object::Callable(crate::native::clock()));

        Self { globals, environment, locals: HashMap::new(), error_reporter: None }
    }

    pub fn with_error_reporting(self, error_reporter: SharedErrorReporter) -> Self {
        Self { error_reporter: Some(error_reporter), ..self }
    }
}
