use std::{collections::HashMap, fmt::Display};

use crate::{prelude::*, runtime_error};

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, LiteralValue>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: LiteralValue) {
        self.values.insert(name.to_owned(), value);
    }

    pub fn assign(&mut self, name: &Token, value: LiteralValue) -> Result<(), RuntimeError> {
        if !self.values.contains_key(&name.lexeme) {
            return Err(RuntimeError::UndefinedVariable {
                name: name.clone(),
                msg: format!("Undefined variable '{}'", name.lexeme),
            });
        }

        self.values.insert(name.lexeme.clone(), value);
        Ok(())
    }

    pub fn get(&self, name: &Token) -> Result<LiteralValue, RuntimeError> {
        let value = self.values.get(&name.lexeme).map(|lit| lit.to_owned());
        value.ok_or_else(move || RuntimeError::UndefinedVariable {
            name: name.clone(),
            msg: format!("Undefined variable '{}'", name.lexeme),
        })
    }
}

pub struct Interpreter {
    environment: Environment,
}

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    InvalidOperand { operator: Token, msg: String },
    UndefinedVariable { name: Token, msg: String },
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::InvalidOperand { operator, msg } => {
                write!(f, "{}\n[line {}]", msg, operator.line)
            }
            RuntimeError::UndefinedVariable { name, msg } => {
                write!(f, "{}\n[line {}]", msg, name.line)
            }
        }
    }
}

type InterpreterResult = Result<LiteralValue, RuntimeError>;

impl Visitor<Expr> for Interpreter {
    type Result = LiteralValue;
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
            Expr::Variable { name } => self.environment.get(name),
            Expr::Assignment { name, value } => {
                let value = self.visit(value.as_ref())?;
                self.environment.assign(&name, value.clone())?;
                Ok(value)
            }
        }
    }
}

impl Visitor<Stmt> for Interpreter {
    type Result = ();
    type Error = RuntimeError;

    fn visit(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Expression { expr } => self.evaluate(expr)?,
            Stmt::Print { expr } => {
                let value = self.evaluate(expr)?;
                println!("{}", value);
                value
            }
            Stmt::Var { name, initializer } => {
                let value = if let Some(expr) = initializer {
                    self.evaluate(expr)?
                } else {
                    LiteralValue::Null
                };

                self.environment.define(&name.lexeme, value);
                LiteralValue::Null
            }
        };
        Ok(())
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }
    pub fn interpret(&mut self, statements: &[Stmt]) {
        for stmt in statements {
            match self.execute(&stmt) {
                Err(e) => runtime_error(e),
                _ => {}
            }
        }
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        self.visit(stmt)
    }

    pub fn evaluate(&mut self, expression: &Expr) -> InterpreterResult {
        self.visit(expression)
    }

    fn is_truthy(&self, value: &LiteralValue) -> bool {
        match value {
            LiteralValue::Null => false,
            LiteralValue::Boolean(false) => false,
            _ => true,
        }
    }

    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> InterpreterResult {
        let value = self.visit(right)?;
        match operator.token_type {
            TokenType::Minus => {
                if let LiteralValue::Number(n) = value {
                    Ok(LiteralValue::Number(-n))
                } else {
                    Err(RuntimeError::InvalidOperand {
                        operator: operator.clone(),
                        msg: "Operand must be a number".to_owned(),
                    })
                }
            }
            TokenType::Bang => Ok(LiteralValue::Boolean(!self.is_truthy(&value))),

            // Unreachable code. We don't have any unary expression except the ones above.
            _ => Ok(LiteralValue::Null),
        }
    }

    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> InterpreterResult {
        let left_value = self.visit(left)?;
        let right_value = self.visit(right)?;

        match operator.token_type {
            TokenType::Plus => {
                if let (Some(l), Some(r)) = (left_value.number(), right_value.number()) {
                    Ok(LiteralValue::Number(l + r))
                } else if let (Some(l), Some(r)) = (left_value.string(), right_value.string()) {
                    Ok(LiteralValue::String(format!("{}{}", l, r)))
                } else {
                    Err(RuntimeError::InvalidOperand {
                        operator: operator.clone(),
                        msg: "Operands must be two numbers or two strings".to_owned(),
                    })
                }
            }
            TokenType::Minus => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| LiteralValue::Number(l - r)),
            TokenType::Star => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| LiteralValue::Number(l * r)),
            TokenType::Slash => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| LiteralValue::Number(l / r)),
            TokenType::Greater => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| LiteralValue::Boolean(l > r)),
            TokenType::GreaterEqual => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| LiteralValue::Boolean(l >= r)),
            TokenType::Less => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| LiteralValue::Boolean(l < r)),
            TokenType::LessEqual => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| LiteralValue::Boolean(l <= r)),

            TokenType::EqualEqual => Ok(LiteralValue::Boolean(left_value == right_value)),
            TokenType::BangEqual => Ok(LiteralValue::Boolean(left_value != right_value)),

            // Unreachable code
            _ => Ok(LiteralValue::Null),
        }
    }

    fn check_number_operands(
        &self,
        operator: &Token,
        left: &LiteralValue,
        right: &LiteralValue,
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
        assert_number!("-3.14", -3.14);
    }

    #[test]
    fn unary_bang() {
        assert_boolean!("!true", false);
        assert_boolean!("!false", true);
    }

    #[test]
    fn binary_plus_numbers() {
        assert_number!("10 + 20", 30.0);
    }

    #[test]
    fn binary_plus_strings() {
        assert_string!(r#" "Hello " + "World!" "#, "Hello World!".to_string());
    }

    #[test]
    fn binary_minus() {
        assert_number!("10 - 20", -10.0);
    }

    #[test]
    fn binary_star() {
        assert_number!("10 * 20", 200.0);
    }

    #[test]
    fn binary_slash() {
        assert_number!("10 / 20", 0.5);
    }

    #[test]
    fn binary_greater() {
        assert_boolean!("10 > 20", false);
        assert_boolean!("20 > 10", true);
    }

    #[test]
    fn binary_greater_equal() {
        assert_boolean!("10 >= 20", false);
        assert_boolean!("20 >= 10", true);
    }

    #[test]
    fn binary_less() {
        assert_boolean!("10 < 20", true);
        assert_boolean!("20 < 10", false);
    }

    #[test]
    fn binary_less_equal() {
        assert_boolean!("10 <= 20", true);
        assert_boolean!("20 <= 10", false);
    }

    #[test]
    fn binary_equal_equal() {
        assert_boolean!("10 == 20", false);
        assert_boolean!("10 == 10", true);
    }

    #[test]
    fn binary_bang_equal() {
        assert_boolean!("10 != 20", true);
        assert_boolean!("10 != 10", false);
    }
}
