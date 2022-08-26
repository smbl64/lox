use std::env;
fn main() {
    let mut args = env::args().into_iter().skip(1).collect::<Vec<_>>();
    if args.len() > 1 {
        println!("Usage: clox [script]");
        std::process::exit(64);
    } else if args.len() == 1 {
        let filename = args.pop().unwrap();
        clox::run_file(filename);
    } else {
        clox::run_prompt();
    }
}
