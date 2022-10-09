#![allow(clippy::new_without_default)]
mod ast;
mod interpreter;
mod parser;
mod scanner;
mod token;

pub mod prelude {
    pub use crate::ast::*;
    pub use crate::interpreter::*;
    pub use crate::parser::*;
    pub use crate::scanner::*;
    pub use crate::token::*;
}

use std::{cell::RefCell, io::Write, rc::Rc};

use prelude::{Interpreter, Parser, Resolver, RuntimeError};

pub struct Lox {
    interpreter: Interpreter,
    error_reporter: SharedErrorReporter,
}

impl Lox {
    pub fn new() -> Self {
        let error_reporter = Rc::new(RefCell::new(ErrorReporter::default()));

        Self {
            interpreter: Interpreter::new().with_error_reporting(error_reporter.clone()),
            error_reporter: error_reporter.clone(),
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
                    // TODO any way to pass this error directly upwards?
                    return Err(anyhow::anyhow!("{}", e));
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
        eprintln!("[line {}] Error {}: {}", line, location, message);
        self.had_error = true;
    }

    pub fn runtime_error(&mut self, e: RuntimeError) {
        eprintln!("{}", e);
        self.had_runtime_error = true;
    }
}

type SharedErrorReporter = Rc<RefCell<ErrorReporter>>;
