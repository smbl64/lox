use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::prelude::*;

impl Interpreter {
    pub fn interpret(&mut self, statements: &[Stmt]) {
        for stmt in statements {
            if let Err(e) = self.execute(stmt) {
                self.runtime_error(e);
            }
        }
    }

    pub fn execute_block<I, R>(
        &mut self,
        statements: I,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<(), RuntimeError>
    where
        I: IntoIterator<Item = R>,
        R: AsRef<Stmt>,
    {
        let prev_env = self.environment.clone();
        self.environment = environment;

        for s in statements {
            let result = self.execute(s.as_ref());
            if matches!(result, Err(_)) {
                self.environment = prev_env;
                return result;
            }
        }

        self.environment = prev_env;
        Ok(())
    }

    pub fn resolve(&mut self, input: &Expr, depth: usize) {
        self.locals.insert(input.unique_id(), depth);
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Expression { expr } => {
                self.evaluate_expr(expr)?;
            }
            Stmt::Class { name, methods, superclass } => {
                self.handle_class_stmt(name, methods, superclass)?
            }
            Stmt::Function { name, params, body } => {
                // self.environment is the current active environment when function
                // is being declared, NOT when it's being called!
                // In other words, this is the enclosing environment in which the function is
                // declarad. For inner functions, it refers to their parent function's
                // environment.
                let env = self.environment.clone();
                let function = LoxFunction::new(name.clone(), params.to_vec(), body, env, false);
                self.environment
                    .borrow_mut()
                    .define(&name.lexeme, Object::Callable(Rc::new(function)));
            }
            Stmt::Break { token } => return Err(RuntimeError::Break { token: token.clone() }),
            Stmt::Return { keyword, value } => {
                let value =
                    if let Some(expr) = value { self.evaluate_expr(expr)? } else { Object::Null };

                return Err(RuntimeError::Return { token: keyword.clone(), value });
            }
            Stmt::Print { exprs } => {
                for expr in exprs {
                    let value = self.evaluate_expr(expr)?;
                    print!("{value}");
                }

                println!();
            }
            Stmt::Var { name, initializer } => {
                let value = if let Some(expr) = initializer {
                    self.evaluate_expr(expr)?
                } else {
                    Object::Null
                };

                self.environment.borrow_mut().define(&name.lexeme, value);
            }
            Stmt::Block { statements } => {
                // Create a new environment for executing the block
                let new_env = Environment::new().with_enclosing(self.environment.clone()).as_rc();

                self.execute_block(statements, new_env)?;
            }
            Stmt::If { condition, then_branch, else_branch } => {
                let condition_result = self.evaluate_expr(condition)?;

                if self.is_truthy(&condition_result) {
                    self.execute(then_branch.as_ref())?;
                } else if let Some(stmt) = else_branch {
                    self.execute(stmt.as_ref())?;
                }
            }
            Stmt::While { condition, body } => self.handle_while_stmt(condition, body)?,
        };
        Ok(())
    }

    pub fn handle_class_stmt(
        &mut self,
        name: &Token,
        methods: &Vec<Stmt>,
        superclass: &Option<Expr>,
    ) -> Result<(), RuntimeError> {
        // TODO: this looks really ugly!!
        let superclass = if let Some(s) = superclass {
            let obj = self.evaluate_expr(s)?;
            match obj {
                Object::Class(c) => Some(c),
                _ => {
                    if let Expr::Variable { name: super_name } = s {
                        return Err(RuntimeError::Generic {
                            name: super_name.clone(),
                            msg: "Superclass must be a class".to_owned(),
                        });
                    } else {
                        panic!("Superclass is not enclosed in a Expr::Variable!");
                    }
                }
            }
        } else {
            None
        };

        self.environment.borrow_mut().define(&name.lexeme, Object::Null);

        if let Some(ref superclass) = superclass {
            self.environment = Environment::new().with_enclosing(self.environment.clone()).as_rc();

            self.environment.borrow_mut().define("super", Object::Class(superclass.clone()));
        }

        // Create method functions
        let mut method_funcs = HashMap::new();
        for method in methods {
            if let Stmt::Function { name, params, body } = method {
                let is_initializer = name.lexeme == "init";

                method_funcs.insert(
                    name.lexeme.clone(),
                    Rc::new(LoxFunction::new(
                        name.clone(),
                        params.to_vec(),
                        body,
                        self.environment.clone(),
                        is_initializer,
                    )),
                );
            } else {
                panic!("Method is not encapsulated in Stmt::Function");
            }
        }

        let class =
            Rc::new(RefCell::new(Class::new(&name.lexeme, method_funcs, superclass.clone())));

        if superclass.is_some() {
            let enclosing = self.environment.borrow().enclosing.clone().unwrap();
            self.environment = enclosing;
        }

        self.environment.borrow_mut().assign(name, Object::Class(class))
    }

    pub fn handle_while_stmt(
        &mut self,
        condition: &Expr,
        body: &Box<Stmt>,
    ) -> Result<(), RuntimeError> {
        loop {
            let value = &self.evaluate_expr(condition)?;
            if !self.is_truthy(value) {
                break;
            }

            // We will catch 'Break' runtime errors. That error means that we hit a `break`
            // statement. Any other error will be propagated up.
            let result = self.execute(body);

            if matches!(result, Err(RuntimeError::Break { token: _ })) {
                break;
            }

            result?;
        }

        Ok(())
    }

    fn runtime_error(&self, e: RuntimeError) {
        if self.error_reporter.is_none() {
            return;
        }
        let reporter = self.error_reporter.as_ref().unwrap();
        let mut reporter = reporter.borrow_mut();
        reporter.runtime_error(&e);
    }
}
