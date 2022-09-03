use rlox::prelude::Scanner;

#[test]
fn scanner_works() {
    let input = "2 and 3";
    let mut scanner = Scanner::new(input);
    let tokens = scanner.scan_tokens();
    //dbg!(&tokens);
    assert_eq!(tokens.len(), 4);
}
