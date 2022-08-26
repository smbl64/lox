use std::env;
fn main() -> Result<(), anyhow::Error> {
    let mut args = env::args().into_iter().skip(1).collect::<Vec<_>>();

    match args.len() {
        1 => {
            let filename = args.pop().unwrap();
            clox::run_file(filename.as_ref())
        }
        2.. => {
            println!("Usage: clox [script]");
            std::process::exit(64);
        }
        _ => clox::run_prompt(),
    }
}
