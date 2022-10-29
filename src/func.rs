use std::fmt::{Debug, Display};
use std::rc::Rc;

use crate::prelude::*;

pub trait Callable: Debug + Display {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpret: &mut Interpreter,
        arguments: &[Object],
    ) -> Result<Object, RuntimeInterrupt>;
}

#[derive(Debug, Clone)]
pub struct LoxFunction {
    name: Token,
    params: Vec<Token>,
    body: Vec<Rc<Stmt>>,
    closure: Shared<Environment>,
    is_initializer: bool,
}

impl LoxFunction {
    pub fn new(
        name: Token,
        params: Vec<Token>,
        body: &[Rc<Stmt>],
        closure: Shared<Environment>,
        is_initializer: bool,
    ) -> Self {
        Self { name, params, body: body.to_vec(), closure, is_initializer }
    }

    pub fn bind(&self, this: Object) -> Rc<LoxFunction> {
        let env = Environment::new().with_enclosing(self.closure.clone()).as_shared();
        env.borrow_mut().define("this", this);

        Rc::new(LoxFunction::new(
            self.name.clone(),
            self.params.clone(),
            &self.body,
            env,
            self.is_initializer,
        ))
    }

    fn new_env_for_call(&self, arguments: &[Object]) -> Shared<Environment> {
        let mut environment = Environment::new().with_enclosing(self.closure.clone());

        // Put all arguments in this new environment
        //let mut env_borrow = environment.borrow_mut();
        for (arg, param) in arguments.iter().zip(&self.params) {
            environment.define(param.lexeme.as_str(), arg.clone());
        }

        environment.as_shared()
    }
}

impl Callable for LoxFunction {
    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(
        &self,
        interpret: &mut Interpreter,
        arguments: &[Object],
    ) -> Result<Object, RuntimeInterrupt> {
        // Every call needs a new environment (i.e. "stack"). If we keep one stack for
        // all calls, subsequent calls will override each others' parameters.
        let environment = self.new_env_for_call(arguments);

        let res = interpret.execute_block(&self.body, environment);

        // If this function is an initializer and we didn't get an error, return "this"
        // as the return value.
        if self.is_initializer
            && (matches!(res, Ok(_)) || matches!(res, Err(RuntimeInterrupt::Return { .. })))
        {
            let token = Token::new(TokenType::This, "this", None, -1);
            return self.closure.borrow().get_at(0, &token);
        }

        // If a 'Return' runtime exception is generated, this means the block had a
        // return statement. We should extract the value from it and return it.
        // Otherwise, return Object::Null or the runtime error.
        if let Err(RuntimeInterrupt::Return { value, .. }) = res {
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
