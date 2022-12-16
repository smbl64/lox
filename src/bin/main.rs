use std::env;

use lox::Lox;

fn main() -> Result<(), anyhow::Error> {
    let mut args = env::args().into_iter().skip(1).collect::<Vec<_>>();

    let mut lox = Lox::new();
    match args.len() {
        1 => {
            let filename = args.pop().unwrap();
            lox.run_file(filename.as_ref())
        }
        _ => {
            let bin_name = env!("CARGO_BIN_NAME");
            println!("Usage: {} [script]", bin_name);
            std::process::exit(64);
        }
    }
}
