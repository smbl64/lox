#![allow(clippy::new_without_default)]
#![allow(clippy::vtable_address_comparisons)]

mod ast;
mod class;
mod environment;
mod error;
mod func;
mod interpreter;
mod native;
mod object;
mod parser;
mod printer;
mod resolver;
mod scanner;
mod token;

pub mod prelude {
    pub use crate::ast::*;
    pub use crate::class::*;
    pub use crate::environment::Environment;
    pub use crate::error::*;
    pub use crate::func::*;
    pub use crate::interpreter::*;
    pub use crate::object::*;
    pub use crate::parser::*;
    pub use crate::resolver::Resolver;
    pub use crate::scanner::*;
    pub use crate::token::*;
    pub use crate::Shared;
}

use std::cell::RefCell;
use std::rc::Rc;

use prelude::{Interpreter, Parser, Resolver, RuntimeInterrupt, TokenType};
use resolver::ResolverError;

pub type Shared<T> = Rc<RefCell<T>>;

pub struct Lox {}

impl Lox {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run_file(&mut self, filename: &str) -> Result<(), anyhow::Error> {
        let content = std::fs::read_to_string(filename)?;

        let mut scanner = scanner::Scanner::new(&content);
        let tokens = match scanner.scan_tokens() {
            Ok(tokens) => tokens,
            Err(errors) => {
                self.print_scanner_errors(errors.as_ref());
                return Ok(());
            }
        };

        let mut parser = Parser::new(tokens);
        let statements = match parser.parse() {
            Ok(stmts) => stmts,
            Err(errors) => {
                self.print_parser_errors(errors.as_ref());
                return Ok(());
            }
        };

        let mut interpreter = Interpreter::new();
        let mut resolver = Resolver::new(&mut interpreter);
        if let Err(errors) = resolver.resolve(&statements) {
            self.print_resolver_errors(errors.as_ref());
            return Ok(());
        }

        if let Err(errors) = interpreter.interpret(&statements) {
            self.print_interpreter_errors(errors.as_ref());
            return Ok(());
        }

        Ok(())
    }

    fn print_scanner_errors(&mut self, errors: &[scanner::ScannerError]) {
        errors.iter().for_each(|e| self.error(e.line, "", &e.message));
    }

    fn print_parser_errors(&mut self, errors: &[parser::ParserError]) {
        for e in errors {
            if e.token.token_type == TokenType::EOF {
                self.error(e.token.line, "at end", &e.message);
            } else {
                self.error(e.token.line, &format!("at '{}'", e.token.lexeme), &e.message);
            }
        }
    }

    fn print_resolver_errors(&mut self, errors: &[ResolverError]) {
        for e in errors {
            eprintln!("{e}");
        }
    }

    fn print_interpreter_errors(&self, errors: &[interpreter::InterpreterError]) {
        for e in errors {
            eprintln!("[line {}] {}", e.line, e.message);
        }
    }

    fn error(&mut self, line: u32, location: &str, message: &str) {
        if location.is_empty() {
            eprintln!("[line {line}] Error: {message}");
        } else {
            eprintln!("[line {line}] Error {location}: {message}");
        }
    }
}
