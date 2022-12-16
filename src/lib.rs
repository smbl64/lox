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

pub type Shared<T> = Rc<RefCell<T>>;

pub struct Lox {
    error_messages: Vec<String>,
}

impl Lox {
    pub fn new() -> Self {
        Self { error_messages: Vec::new() }
    }

    pub fn run_file(&mut self, filename: &str) -> Result<(), anyhow::Error> {
        let content = std::fs::read_to_string(filename)?;

        let tokens = self.scan(content)?;
        let statements = self.parse(tokens)?;

        let mut interpreter = Interpreter::new();
        let mut resolver = Resolver::new(&mut interpreter);
        if let Err(errors) = resolver.resolve(&statements) {
            for e in errors {
                self.error_messages.push(format!("{e}"));
            }
            return Err(self.aggregate_errors());
        }

        if let Err(errors) = interpreter.interpret(&statements) {
            for e in errors {
                self.error_messages.push(format!("[line {}] {}", e.line, e.message));
            }
            return Err(self.aggregate_errors());
        }

        Ok(())
    }

    fn parse(&mut self, tokens: Vec<prelude::Token>) -> Result<Vec<prelude::Stmt>, anyhow::Error> {
        let mut parser = Parser::new(tokens);
        parser.parse().or_else(|errors| {
            self.add_parse_errors(errors);
            return Err(self.aggregate_errors());
        })
    }

    fn add_parse_errors(&mut self, errors: Vec<prelude::ParserError>) {
        for e in errors {
            if e.token.token_type == TokenType::EOF {
                self.add_error(e.token.line, "at end", &e.message);
            } else {
                self.add_error(e.token.line, &format!("at '{}'", e.token.lexeme), &e.message);
            }
        }
    }

    fn scan(&mut self, content: String) -> Result<Vec<prelude::Token>, anyhow::Error> {
        let mut scanner = scanner::Scanner::new(&content);
        scanner.scan_tokens().or_else(|errors| {
            errors.iter().for_each(|e| self.add_error(e.line, "", &e.message));
            return Err(self.aggregate_errors());
        })
    }

    fn aggregate_errors(&mut self) -> anyhow::Error {
        let res = anyhow::anyhow!(self.error_messages.join("\n"));
        self.error_messages.clear();
        res
    }

    fn add_error(&mut self, line: u32, location: &str, message: &str) {
        if location.is_empty() {
            self.error_messages.push(format!("[line {line}] Error: {message}"));
        } else {
            self.error_messages.push(format!("[line {line}] Error {location}: {message}"));
        }
    }
}
