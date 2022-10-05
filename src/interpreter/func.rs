use std::fmt::Display;

use crate::{
    ast::Stmt,
    token::{Callable, Object, Token},
};

use super::{environment::Environment, Interpreter};

#[derive(Debug)]
pub struct LoxFunction {
    name: Token,
    params: Vec<Token>,
    body: Vec<Stmt>,
}

impl LoxFunction {
    pub fn new(declaration: Stmt) -> Self {
        match declaration {
            Stmt::Function { name, params, body } => Self { name, params, body },
            // TODO: Find a way to enforce it at compile time!
            _ => panic!("Only Stmt::Function is allowd!"),
        }
    }
}

impl Callable for LoxFunction {
    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(&self, interpret: &mut Interpreter, arguments: Vec<Object>) -> Object {
        let mut environment = Environment::with_enclosing(interpret.globals.clone());
        for (arg, param) in arguments.iter().zip(&self.params) {
            environment.define(param.lexeme.as_str(), arg.clone());
        }

        let _ = interpret.execute_block(&self.body);
        Object::Null
    }
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name.lexeme)
    }
}
