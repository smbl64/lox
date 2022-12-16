use lox::prelude::Scanner;

#[test]
fn scanner_works() {
    let input = "2 and 3";
    let mut scanner = Scanner::new(input);
    let (tokens, _) = scanner.scan_tokens();
    assert_eq!(tokens.len(), 4);
}

#[test]
fn can_read_unicode() {
    let input = r"
        // Latin 1 Supplement: £§¶ÜÞ
        // Latin Extended-A: ĐĦŋœ
        // Latin Extended-B: ƂƢƩǁ
        // Emoji: 🦀
        2 and 3
    ";
    let mut scanner = Scanner::new(input);
    let (tokens, _) = scanner.scan_tokens();
    assert_eq!(tokens.len(), 4);
}
