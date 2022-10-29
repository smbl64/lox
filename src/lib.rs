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
use std::io::Write;
use std::rc::Rc;

use prelude::{Interpreter, Parser, Resolver, RuntimeInterrupt};
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

    pub fn run_prompt(&mut self) -> Result<(), anyhow::Error> {
        let reader = std::io::stdin();

        loop {
            print!("> ");
            std::io::stdout().flush().expect("failed to flush stdout");

            let mut line = String::new();
            let n = reader.read_line(&mut line)?;
            if n == 0 {
                return Ok(());
            }
            let line = line.trim_end();
            // Ignore errors in the prompt mode
            let _ = self.run(line);
        }
    }

    pub fn run(&mut self, input: &str) -> Result<(), anyhow::Error> {
        let mut scanner =
            scanner::Scanner::new(input).with_error_reporting(self.error_reporter.clone());
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens).with_error_reporting(self.error_reporter.clone());
        match parser.parse() {
            None => return Ok(()),
            Some(stmts) => {
                let mut resolver = Resolver::new(&mut self.interpreter);
                if let Err(e) = resolver.resolve(&stmts) {
                    self.error_reporter.borrow_mut().resolver_error(&e);
                    return Ok(());
                }

                self.interpreter.interpret(&stmts);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ErrorReporter {
    pub had_error: bool,
    pub had_runtime_error: bool,
}

impl ErrorReporter {
    pub fn error(&mut self, line: i32, message: &str) {
        self.report(line, "", message);
    }

    pub fn report(&mut self, line: i32, location: &str, message: &str) {
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
