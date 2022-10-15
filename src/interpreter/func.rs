use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::prelude::*;

#[derive(Debug)]
pub struct LoxFunction {
    name: Token,
    params: Vec<Token>,
    body: Vec<Rc<Stmt>>,
    closure: Rc<RefCell<Environment>>,
}

impl LoxFunction {
    pub fn new(
        name: Token,
        params: Vec<Token>,
        body: &[Rc<Stmt>],
        closure: Rc<RefCell<Environment>>,
    ) -> Self {
        Self {
            name,
            params,
            body: body.to_vec(),
            closure,
        }
    }
}

impl Callable for LoxFunction {
    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(
        &self,
        interpret: &mut Interpreter,
        arguments: Vec<Object>,
    ) -> Result<Object, RuntimeError> {
        let mut environment = Environment::with_enclosing(self.closure.clone());
        for (arg, param) in arguments.iter().zip(&self.params) {
            environment.define(param.lexeme.as_str(), arg.clone());
        }

        let environment = Rc::new(RefCell::new(environment));
        let res = interpret.execute_block(&self.body, environment);

        // If a 'Return' runtime exception is generated, this means the block had a
        // return statement. We should extract the value from it and return it.
        // Otherwise, return Object::Null or the runtime error.
        if let Err(RuntimeError::Return { token: _, value }) = res {
            Ok(value)
        } else {
            res.map(|_| Object::Null)
        }
    }
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name.lexeme)
    }
}
