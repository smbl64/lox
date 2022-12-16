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
pub type SharedErrorReporter = Shared<ErrorReporter>;

pub struct Lox {
    interpreter: Interpreter,
    error_reporter: SharedErrorReporter,
}

impl Lox {
    pub fn new() -> Self {
        let error_reporter = Rc::new(RefCell::new(ErrorReporter::default()));

        Self {
            interpreter: Interpreter::new().with_error_reporting(error_reporter.clone()),
            error_reporter,
        }
    }
}

impl Lox {
    pub fn run_file(&mut self, filename: &str) -> Result<(), anyhow::Error> {
        let content = std::fs::read_to_string(filename)?;
        self.run(content.as_ref())
        // TODO
        //// Indicate an error in the exit code.
        //if (hadError) System.exit(65);
        //if (hadRuntimeError) System.exit(70);
    }

    fn run(&mut self, input: &str) -> Result<(), anyhow::Error> {
        let mut scanner = scanner::Scanner::new(input);

        let tokens = match scanner.scan_tokens() {
            Ok(tokens) => tokens,
            Err(errors) => {
                self.print_scanner_errors(errors);
                return Ok(());
            }
        };

        let mut parser = Parser::new(tokens);
        let statements = match parser.parse() {
            Ok(stmts) => stmts,
            Err(errors) => {
                self.print_parser_errors(errors);
                return Ok(());
            }
        };

        if self.error_reporter.borrow().had_error {
            return Ok(());
        }

        let mut resolver = Resolver::new(&mut self.interpreter);
        if let Err(errors) = resolver.resolve(&statements) {
            for e in errors {
                self.error_reporter.borrow_mut().resolver_error(&e);
            }
            return Ok(());
        }

        self.interpreter.interpret(&statements);

        Ok(())
    }

    fn print_scanner_errors(&mut self, errors: Vec<scanner::ScannerError>) {
        let mut reporter = self.error_reporter.borrow_mut();
        errors.iter().for_each(|e| reporter.error(e.line, &e.message));
    }

    fn print_parser_errors(&mut self, errors: Vec<parser::ParserError>) {
        let mut reporter = self.error_reporter.borrow_mut();

        for e in errors {
            if e.token.token_type == TokenType::EOF {
                reporter.report(e.token.line, "at end", &e.message);
            } else {
                reporter.report(e.token.line, &format!("at '{}'", e.token.lexeme), &e.message);
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct ErrorReporter {
    pub had_error: bool,
    pub had_runtime_error: bool,
}

impl ErrorReporter {
    pub fn error(&mut self, line: u32, message: &str) {
        self.report(line, "", message);
    }

    pub fn report(&mut self, line: u32, location: &str, message: &str) {
        if location.is_empty() {
            eprintln!("[line {line}] Error: {message}");
        } else {
            eprintln!("[line {line}] Error {location}: {message}");
        }

        self.had_error = true;
    }

    pub fn runtime_error(&mut self, e: &RuntimeInterrupt) {
        eprintln!("{e}");
        self.had_runtime_error = true;
    }

    pub fn resolver_error(&mut self, e: &ResolverError) {
        eprintln!("{e}");
        self.had_error = true;
    }
}
