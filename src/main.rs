use lox::Lox;
use std::env;

fn main() -> Result<(), anyhow::Error> {
    let mut args = env::args().into_iter().skip(1).collect::<Vec<_>>();

    let mut lox = Lox::new();
    match args.len() {
        1 => {
            let filename = args.pop().unwrap();
            lox.run_file(filename.as_ref())
        }
        2.. => {
            println!("Usage: rlox [script]");
            std::process::exit(64);
        }
        _ => lox.run_prompt(),
    }
}
