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
        let globals = Environment::new().as_rc();
        let environment = globals.clone();

        globals.borrow_mut().define("clock", Object::Callable(crate::native::clock()));

        Self { globals, environment, locals: HashMap::new(), error_reporter: None }
    }

    pub fn with_error_reporting(self, error_reporter: SharedErrorReporter) -> Self {
        Self { error_reporter: Some(error_reporter), ..self }
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
