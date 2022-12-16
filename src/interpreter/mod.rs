mod expr;
mod stmt;

use std::collections::HashMap;

use crate::prelude::*;

type InterpreterResult = Result<Object, RuntimeInterrupt>;

pub struct InterpreterError {
    pub line: u32,
    pub message: String,
}

pub struct Interpreter {
    pub globals: Shared<Environment>,
    environment: Shared<Environment>,
    locals: HashMap<UniqueId, usize>, // unique id -> depth
    errors: Vec<InterpreterError>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Environment::new().as_shared();
        let environment = globals.clone();

        globals.borrow_mut().define("clock", Object::Callable(crate::native::clock()));

        Self { globals, environment, locals: HashMap::new(), errors: Vec::new() }
    }
}
