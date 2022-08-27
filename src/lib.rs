mod scanner;
use std::io::Write;

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
        run(line)?;
    }
}

pub fn run(input: &str) -> Result<(), anyhow::Error> {
    let scanner = scanner::Scanner::new(input);
    let tokens = scanner.scan_tokens();

    for token in tokens {
        println!("{:?}", token);
    }
    Ok(())
}
