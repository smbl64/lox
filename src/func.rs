use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct LoxFunction {
    name: Token,
    params: Vec<Token>,
    body: Vec<Rc<Stmt>>,
    closure: Rc<RefCell<Environment>>,
    is_initializer: bool,
}

impl LoxFunction {
    pub fn new(
        name: Token,
        params: Vec<Token>,
        body: &[Rc<Stmt>],
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> Self {
        Self { name, params, body: body.to_vec(), closure, is_initializer }
    }

    pub fn bind(&self, instance: Object) -> Rc<LoxFunction> {
        let env = Environment::new().with_enclosing(self.closure.clone()).as_rc();
        env.borrow_mut().define("this", instance);

        Rc::new(LoxFunction::new(
            self.name.clone(),
            self.params.clone(),
            &self.body,
            env,
            self.is_initializer,
        ))
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
        let environment = Environment::new().with_enclosing(self.closure.clone()).as_rc();

        {
            let mut env_borrow = environment.borrow_mut();
            for (arg, param) in arguments.iter().zip(&self.params) {
                env_borrow.define(param.lexeme.as_str(), arg.clone());
            }
        }

        let res = interpret.execute_block(&self.body, environment);

        // If this is an initializer and we didn't get an error, return "this" as the
        // return value
        if self.is_initializer
            && (matches!(res, Ok(_))
                || matches!(res, Err(RuntimeError::Return { token: _, value: _ })))
        {
            let token = Token::new(TokenType::This, "this", None, -1);
            return self.closure.borrow().get_at(0, &token);
        }

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
