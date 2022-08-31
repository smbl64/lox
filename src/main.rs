use std::env;
fn main() -> Result<(), anyhow::Error> {
    let mut args = env::args().into_iter().skip(1).collect::<Vec<_>>();

    match args.len() {
        1 => {
            let filename = args.pop().unwrap();
            rlox::run_file(filename.as_ref())
        }
        2.. => {
            println!("Usage: rlox [script]");
            std::process::exit(64);
        }
        _ => rlox::run_prompt(),
    }
}
