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

use std::io::Write;

use prelude::{Interpreter, Parser, Resolver, RuntimeError};

pub struct Lox {
    pub had_error: bool,
    pub had_runtime_error: bool,
    interpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Self {
        Self {
            had_error: false,
            had_runtime_error: false,
            interpreter: Interpreter::new(),
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
        let mut scanner = scanner::Scanner::new(input);
        let tokens = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
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

fn error(line: i32, message: &str) {
    report(line, "", message);
}

fn report(line: i32, location: &str, message: &str) {
    eprintln!("[line {}] Error {}: {}", line, location, message);
    // TODO
    //get_lox().had_error = true;
}

fn runtime_error(e: RuntimeError) {
    eprintln!("{}", e);
    // TODO
    //get_lox().had_runtime_error = true;
}

trait ErrorReporter {
    fn error(line: i32, message: &str);
    fn report(line: i32, location: &str, message: &str);
    fn runtime_error(e: RuntimeError);
}
