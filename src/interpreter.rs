use std::cell::RefCell;
use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::{prelude::*, runtime_error};

#[derive(Debug)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        Self::with_enclosing(None)
    }

    pub fn with_enclosing(enclosing: Option<Rc<RefCell<Environment>>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing,
        }
    }

    pub fn define(&mut self, name: &str, value: Object) {
        self.values.insert(name.to_owned(), value);
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<(), RuntimeError> {
        if !self.values.contains_key(&name.lexeme) {
            // Ask one level above if possible
            if let Some(ref e) = self.enclosing {
                return e.borrow_mut().assign(name, value);
            }

            return Err(RuntimeError::UndefinedVariable {
                name: name.clone(),
                msg: format!("Undefined variable '{}'", name.lexeme),
            });
        }

        self.values.insert(name.lexeme.clone(), value);
        Ok(())
    }

    pub fn get(&self, name: &Token) -> Result<Object, RuntimeError> {
        let value = self.values.get(&name.lexeme).map(|lit| lit.to_owned());
        // Ask one level above if possible
        if value.is_none() && self.enclosing.is_some() {
            let rc = self.enclosing.as_ref().unwrap();
            return rc.borrow_mut().get(name);
        }

        value.ok_or_else(move || RuntimeError::UndefinedVariable {
            name: name.clone(),
            msg: format!("Undefined variable '{}'", name.lexeme),
        })
    }
}

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    InvalidOperand { operator: Token, msg: String },
    UndefinedVariable { name: Token, msg: String },
    Break { token: Token },
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::InvalidOperand { operator, msg } => {
                write!(f, "[line {}] {}", operator.line, msg)
            }
            RuntimeError::UndefinedVariable { name, msg } => {
                write!(f, "[line {}] {}", name.line, msg)
            }
            RuntimeError::Break { token } => {
                write!(f, "[line {}] Unexpected break statement", token.line)
            }
        }
    }
}

type InterpreterResult = Result<Object, RuntimeError>;

impl Visitor<Expr> for Interpreter {
    type Result = Object;
    type Error = RuntimeError;

    fn visit(&mut self, expr: &Expr) -> InterpreterResult {
        match expr {
            Expr::Literal { value } => Ok(value.clone()),
            Expr::Grouping { expr: inner } => self.visit(inner.as_ref()),
            Expr::Unary { operator, right } => self.visit_unary(operator, right),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.visit_binary(left, operator, right),
            Expr::Variable { name } => self.environment.borrow().get(name),
            Expr::Assignment { name, value } => {
                let value = self.visit(value.as_ref())?;
                self.environment.borrow_mut().assign(name, value.clone())?;
                Ok(value)
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left_val = self.evaluate(left)?;

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

                self.evaluate(right)
            }
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.evaluate(&callee)?;
                let mut args = vec![];
                for arg in arguments {
                    args.push(self.evaluate(arg)?);
                }

                todo!()
            }
        }
    }
}

impl Visitor<Stmt> for Interpreter {
    type Result = ();
    type Error = RuntimeError;

    fn visit(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Expression { expr } => {
                self.evaluate(expr)?;
            }
            Stmt::Break { token } => {
                return Err(RuntimeError::Break {
                    token: token.clone(),
                })
            }
            Stmt::Print { exprs } => {
                for expr in exprs {
                    let value = self.evaluate(expr)?;
                    print!("{} ", value);
                }

                println!();
            }
            Stmt::Var { name, initializer } => {
                let value = if let Some(expr) = initializer {
                    self.evaluate(expr)?
                } else {
                    Object::Null
                };

                self.environment.borrow_mut().define(&name.lexeme, value);
            }
            Stmt::Block { statements } => {
                let prev_env = self.environment.clone();

                // Create a new environment for executing the block
                let new_env = Environment::with_enclosing(Some(self.environment.clone()));
                self.environment = Rc::new(RefCell::new(new_env));

                let result = self.execute_block(statements);

                // Restore the original environment
                self.environment = prev_env;

                result?;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition_result = self.evaluate(condition)?;

                if self.is_truthy(&condition_result) {
                    self.execute(then_branch.as_ref())?;
                } else if let Some(stmt) = else_branch {
                    self.execute(stmt.as_ref())?;
                }
            }
            Stmt::While { condition, body } => loop {
                let value = &self.evaluate(&condition)?;
                if !self.is_truthy(value) {
                    break;
                }

                // We will catch 'Break' runtime errors. That error means that we hit a `break`
                // statement. Any other error will be propagated up.
                let result = self.execute(&body);

                if matches!(result, Err(RuntimeError::Break { token: _ })) {
                    break;
                }

                result?;
            },
        };
        Ok(())
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) {
        for stmt in statements {
            if let Err(e) = self.execute(stmt) {
                runtime_error(e)
            }
        }
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        self.visit(stmt)
    }

    fn execute_block(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        for s in statements {
            self.execute(s)?;
        }

        Ok(())
    }

    pub fn evaluate(&mut self, expression: &Expr) -> InterpreterResult {
        self.visit(expression)
    }

    fn is_truthy(&self, value: &Object) -> bool {
        !matches!(value, Object::Null | Object::Boolean(false))
    }

    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> InterpreterResult {
        let value = self.visit(right)?;
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

    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> InterpreterResult {
        let left_value = self.visit(left)?;
        let right_value = self.visit(right)?;

        match operator.token_type {
            TokenType::Plus => {
                if let (Some(l), Some(r)) = (left_value.number(), right_value.number()) {
                    Ok(Object::Number(l + r))
                } else if let (Some(l), Some(r)) = (left_value.string(), right_value.string()) {
                    Ok(Object::String(format!("{}{}", l, r)))
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
            let res = ipr.evaluate(&expr);
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), $lit_type($expected));
        };
    }

    macro_rules! assert_number {
        ($source:literal, $expected:expr) => {
            assert_literal!($source, $expected, LiteralValue::Number);
        };
    }

    macro_rules! assert_string {
        ($source:literal, $expected:expr) => {
            assert_literal!($source, $expected, LiteralValue::String);
        };
    }

    macro_rules! assert_boolean {
        ($source:literal, $expected:expr) => {
            assert_literal!($source, $expected, LiteralValue::Boolean);
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
