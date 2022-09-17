#![warn(clippy::unwrap_in_result, clippy::expect_used)]
mod ast;
mod interpreter;
mod scanner;
mod token;
pub mod prelude {
    pub use crate::ast::*;
    pub use crate::interpreter::*;
    pub use crate::scanner::*;
    pub use crate::token::*;
}

use std::io::Write;

use prelude::{Interpreter, Parser, RuntimeError};

static mut LOX: Lox = Lox::new();

// TODO: get rid of this!
pub fn get_lox() -> &'static mut Lox {
    unsafe {
        return &mut LOX;
    }
}

pub struct Lox {
    pub had_error: bool,
    pub had_runtime_error: bool,
}

impl Lox {
    pub const fn new() -> Self {
        Self {
            had_error: false,
            had_runtime_error: false,
        }
    }
}

pub fn run_file(filename: &str) -> Result<(), anyhow::Error> {
    let content = std::fs::read_to_string(filename)?;
    let mut interpreter = Interpreter::new();
    run(content.as_ref(), &mut interpreter)
    // TODO
    //// Indicate an error in the exit code.
    //if (hadError) System.exit(65);
    //if (hadRuntimeError) System.exit(70);
}

pub fn run_prompt() -> Result<(), anyhow::Error> {
    let reader = std::io::stdin();
    let mut interpreter = Interpreter::new();

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
        let _ = run(line, &mut interpreter);
    }
}

pub fn run(input: &str, interpreter: &mut Interpreter) -> Result<(), anyhow::Error> {
    let mut scanner = scanner::Scanner::new(input);
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        None => return Ok(()),
        Some(expr) => {
            interpreter.interpret(&expr);
        }
    }

    Ok(())
}

fn error(line: i32, message: &str) {
    report(line, "", message);
}

fn report(line: i32, location: &str, message: &str) {
    eprintln!("[line {}] Error {}: {}", line, location, message);
    get_lox().had_error = true;
}

fn runtime_error(e: RuntimeError) {
    eprintln!("{}", e);

    get_lox().had_runtime_error = true;
}

trait ErrorReporter {
    fn error(line: i32, message: &str);
    fn report(line: i32, location: &str, message: &str);
    fn runtime_error(e: RuntimeError);
}
