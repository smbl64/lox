use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::func::LoxFunction;
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
        let globals = Environment::new().as_rc();
        let environment = globals.clone();

        globals.borrow_mut().define("clock", Object::Callable(super::native::clock()));

        Self { globals, environment, locals: HashMap::new(), error_reporter: None }
    }

    pub fn with_error_reporting(self, error_reporter: SharedErrorReporter) -> Self {
        Self { error_reporter: Some(error_reporter), ..self }
    }
}

impl Interpreter {
    pub fn evaluate_expr(&mut self, expr: &Expr) -> InterpreterResult {
        match expr {
            Expr::Literal { value } => Ok(value.clone()),
            Expr::Grouping { expr: inner } => self.evaluate_expr(inner.as_ref()),
            Expr::Unary { operator, right } => self.evaluate_unary(operator, right),
            Expr::Binary { left, operator, right } => self.evaluate_binary(left, operator, right),
            Expr::Variable { name } => self.lookup_variable(name, expr),
            Expr::Assignment { name, value } => {
                let value = self.evaluate_expr(value.as_ref())?;

                if let Some(&distance) = self.locals.get(&expr.unique_id()) {
                    self.environment.borrow_mut().assign_at(distance, name, value.clone())?;
                } else {
                    self.globals.borrow_mut().assign(name, value.clone())?;
                }

                Ok(value)
            }
            Expr::Get { object, name } => {
                let object = self.evaluate_expr(object)?;
                if let Object::Instance(ref instance) = object {
                    instance.borrow().get(name, &object)
                } else {
                    Err(RuntimeError::InvalidOperand {
                        operator: name.clone(),
                        msg: "Only instances have properties".to_owned(),
                    })
                }
            }
            Expr::Set { object, name, value } => {
                let value = self.evaluate_expr(value)?;
                let object = self.evaluate_expr(object)?;

                if let Object::Instance(instance) = object {
                    instance.borrow_mut().set(name, value.clone());
                    Ok(value)
                } else {
                    Err(RuntimeError::InvalidOperand {
                        operator: name.clone(),
                        msg: "Only instances have properties".to_owned(),
                    })
                }
            }
            Expr::Super { keyword, method: method_name } => {
                let distance = *self.locals.get(&expr.unique_id()).expect("Cannot find distance");

                let superclass = self.environment.borrow().get_at(distance, keyword)?;
                let superclass = match superclass {
                    Object::Class(c) => c,
                    _ => panic!("Superclass is not wrapped in Object::Class"),
                };

                let this = Token::new(TokenType::Identifier, "this", None, -1);
                let instance = self.environment.borrow().get_at(distance - 1, &this)?;

                let method = superclass.borrow().find_method(&method_name.lexeme);

                if let Some(method) = method {
                    Ok(Object::Callable(method.bind(instance)))
                } else {
                    Err(RuntimeError::UndefinedVariable {
                        name: method_name.clone(),
                        msg: format!("Undefined property '{}'", method_name.lexeme),
                    })
                }
            }
            Expr::This { keyword } => self.lookup_variable(keyword, expr),
            Expr::Logical { left, operator, right } => {
                let left_val = self.evaluate_expr(left)?;

                if operator.token_type == TokenType::Or {
                    if self.is_truthy(&left_val) {
                        return Ok(left_val);
                    }
                } else {
                    // TokenType::And
                    if !self.is_truthy(&left_val) {
                        return Ok(left_val);
                    }
                }

                self.evaluate_expr(right)
            }
            Expr::Call { callee, paren, arguments } => {
                let callee = self.evaluate_expr(callee)?;
                match callee {
                    Object::Callable(callable) => {
                        if callable.arity() != arguments.len() {
                            return Err(RuntimeError::InvalidOperand {
                                operator: paren.clone(),
                                msg: format!(
                                    "Expected {} arguments, got {}",
                                    callable.arity(),
                                    arguments.len()
                                ),
                            });
                        }
                        // Evaluate all arguments
                        let mut args = vec![];
                        for arg in arguments {
                            args.push(self.evaluate_expr(arg)?);
                        }

                        callable.call(self, args)
                    }
                    Object::Class(class) => {
                        let arity = class.borrow().arity();
                        if arity != arguments.len() {
                            return Err(RuntimeError::InvalidOperand {
                                operator: paren.clone(),
                                msg: format!(
                                    "Expected {} arguments, got {}",
                                    arity,
                                    arguments.len()
                                ),
                            });
                        }

                        // Evaluate all arguments
                        let mut args = vec![];
                        for arg in arguments {
                            args.push(self.evaluate_expr(arg)?);
                        }

                        Class::construct(class, args, self).map(Object::Instance)
                    }
                    _ => Err(RuntimeError::InvalidOperand {
                        operator: paren.clone(),
                        msg: "Can only call functions and classes".to_owned(),
                    }),
                }
            }
        }
    }

    fn is_truthy(&self, value: &Object) -> bool {
        !matches!(value, Object::Null | Object::Boolean(false))
    }

    fn evaluate_unary(&mut self, operator: &Token, right: &Expr) -> InterpreterResult {
        let value = self.evaluate_expr(right)?;
        match operator.token_type {
            TokenType::Minus => {
                if let Object::Number(n) = value {
                    Ok(Object::Number(-n))
                } else {
                    Err(RuntimeError::InvalidOperand {
                        operator: operator.clone(),
                        msg: "Operand must be a number".to_owned(),
                    })
                }
            }
            TokenType::Bang => Ok(Object::Boolean(!self.is_truthy(&value))),

            // Unreachable code. We don't have any unary expression except the ones above.
            _ => Ok(Object::Null),
        }
    }

    fn evaluate_binary(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> InterpreterResult {
        let left_value = self.evaluate_expr(left)?;
        let right_value = self.evaluate_expr(right)?;

        match operator.token_type {
            TokenType::Plus => {
                if let (Some(l), Some(r)) = (left_value.number(), right_value.number()) {
                    Ok(Object::Number(l + r))
                } else if let (Some(l), Some(r)) = (left_value.string(), right_value.string()) {
                    Ok(Object::String(format!("{l}{r}")))
                } else {
                    Err(RuntimeError::InvalidOperand {
                        operator: operator.clone(),
                        msg: "Operands must be two numbers or two strings".to_owned(),
                    })
                }
            }
            TokenType::Minus => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| Object::Number(l - r)),
            TokenType::Star => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| Object::Number(l * r)),
            TokenType::Slash => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| Object::Number(l / r)),
            TokenType::Greater => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| Object::Boolean(l > r)),
            TokenType::GreaterEqual => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| Object::Boolean(l >= r)),
            TokenType::Less => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| Object::Boolean(l < r)),
            TokenType::LessEqual => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| Object::Boolean(l <= r)),

            TokenType::EqualEqual => Ok(Object::Boolean(left_value == right_value)),
            TokenType::BangEqual => Ok(Object::Boolean(left_value != right_value)),

            // Unreachable code
            _ => Ok(Object::Null),
        }
    }

    fn check_number_operands(
        &self,
        operator: &Token,
        left: &Object,
        right: &Object,
    ) -> Result<(f64, f64), RuntimeError> {
        if let (Some(l), Some(r)) = (left.number(), right.number()) {
            Ok((l, r))
        } else {
            Err(RuntimeError::InvalidOperand {
                operator: operator.clone(),
                msg: "Operands must be numbers".to_owned(),
            })
        }
    }

    fn lookup_variable(&self, name: &Token, expr: &Expr) -> Result<Object, RuntimeError> {
        if let Some(&distance) = self.locals.get(&expr.unique_id()) {
            self.environment.borrow().get_at(distance, name)
        } else {
            self.globals.borrow().get(name)
        }
    }
}

impl Interpreter {
    pub fn interpret(&mut self, statements: &[Stmt]) {
        for stmt in statements {
            if let Err(e) = self.execute(stmt) {
                self.runtime_error(e);
            }
        }
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        self.evaluate_stmt(stmt)
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

    fn evaluate_stmt(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Expression { expr } => {
                self.evaluate_expr(expr)?;
            }
            Stmt::Class { name, methods, superclass } => {
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
                    self.environment =
                        Environment::new().with_enclosing(self.environment.clone()).as_rc();

                    self.environment
                        .borrow_mut()
                        .define("super", Object::Class(superclass.clone()));
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

                let class = Rc::new(RefCell::new(Class::new(
                    &name.lexeme,
                    method_funcs,
                    superclass.clone(),
                )));

                if superclass.is_some() {
                    let enclosing = self.environment.borrow().enclosing.clone().unwrap();
                    self.environment = enclosing;
                }

                self.environment.borrow_mut().assign(name, Object::Class(class))?;
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
            Stmt::While { condition, body } => loop {
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
            },
        };
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_expression(source: &'static str) -> Expr {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let stmt = parser
            .parse()
            .expect("failed to parse the source")
            .pop()
            .expect("no statement was created");

        match stmt {
            Stmt::Expression { expr } => expr,
            _ => panic!("statement is not an expression"),
        }
    }

    macro_rules! assert_literal {
        ($source:literal, $expected:expr, $lit_type:path) => {
            let mut ipr = Interpreter::new();
            let expr = make_expression($source);
            let res = ipr.evaluate_expr(&expr);
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), $lit_type($expected));
        };
    }

    macro_rules! assert_number {
        ($source:literal, $expected:expr) => {
            assert_literal!($source, $expected, Object::Number);
        };
    }

    macro_rules! assert_string {
        ($source:literal, $expected:expr) => {
            assert_literal!($source, $expected, Object::String);
        };
    }

    macro_rules! assert_boolean {
        ($source:literal, $expected:expr) => {
            assert_literal!($source, $expected, Object::Boolean);
        };
    }

    #[test]
    fn unary_minus() {
        assert_number!("-3.14;", -3.14);
    }

    #[test]
    fn unary_bang() {
        assert_boolean!("!true;", false);
        assert_boolean!("!false;", true);
    }

    #[test]
    fn binary_plus_numbers() {
        assert_number!("10 + 20;", 30.0);
    }

    #[test]
    fn binary_plus_strings() {
        assert_string!(r#" "Hello " + "World!"; "#, "Hello World!".to_string());
    }

    #[test]
    fn binary_minus() {
        assert_number!("10 - 20;", -10.0);
    }

    #[test]
    fn binary_star() {
        assert_number!("10 * 20;", 200.0);
    }

    #[test]
    fn binary_slash() {
        assert_number!("10 / 20;", 0.5);
    }

    #[test]
    fn binary_greater() {
        assert_boolean!("10 > 20;", false);
        assert_boolean!("20 > 10;", true);
    }

    #[test]
    fn binary_greater_equal() {
        assert_boolean!("10 >= 20;", false);
        assert_boolean!("20 >= 10;", true);
    }

    #[test]
    fn binary_less() {
        assert_boolean!("10 < 20;", true);
        assert_boolean!("20 < 10;", false);
    }

    #[test]
    fn binary_less_equal() {
        assert_boolean!("10 <= 20;", true);
        assert_boolean!("20 <= 10;", false);
    }

    #[test]
    fn binary_equal_equal() {
        assert_boolean!("10 == 20;", false);
        assert_boolean!("10 == 10;", true);
    }

    #[test]
    fn binary_bang_equal() {
        assert_boolean!("10 != 20;", true);
        assert_boolean!("10 != 10;", false);
    }
}
