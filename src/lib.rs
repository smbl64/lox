#![warn(clippy::unwrap_in_result, clippy::expect_used)]
mod ast;
mod scanner;
mod token;
pub mod prelude {
    pub use crate::ast::*;
    pub use crate::scanner::*;
    pub use crate::token::*;
}

use std::io::Write;

use prelude::{AstPrinter, Parser, Visitor};

pub fn run_file(filename: &str) -> Result<(), anyhow::Error> {
    let content = std::fs::read_to_string(filename)?;
    run(content.as_ref())
}

pub fn run_prompt() -> Result<(), anyhow::Error> {
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
        let _ = run(line);
    }
}

pub fn run(input: &str) -> Result<(), anyhow::Error> {
    let mut scanner = scanner::Scanner::new(input);
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        None => return Ok(()),
        Some(expr) => {
            let printer = AstPrinter;
            println!("{}", printer.visit(&expr));
        }
    }

    Ok(())
}

pub fn error(line: i32, message: &str) {
    report(line, "", message);
}

fn report(line: i32, location: &str, message: &str) {
    eprintln!("[line {}] Error {}: {}", line, location, message);
    // TODO: HAD_ERROR = true;
}
