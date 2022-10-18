use crate::{prelude::*, SharedErrorReporter};

#[derive(Debug)]
pub struct Scanner {
    source_chars: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: i32,
    error_reporter: Option<SharedErrorReporter>,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Self {
            source_chars: source.chars().collect(),
            start: 0,
            current: 0,
            line: 1,
            tokens: Vec::new(),
            error_reporter: None,
        }
    }

    pub fn with_error_reporting(self, error_reporter: SharedErrorReporter) -> Self {
        Self {
            error_reporter: Some(error_reporter),
            ..self
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::EOF, "", None, self.line));

        // Take our temporary tokens out. It will be replaced by the default()
        // value for the vector
        std::mem::take(&mut self.tokens)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source_chars.len()
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let token_type = if self.match_next('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token_type);
            }
            '=' => {
                let token_type = if self.match_next('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token_type);
            }
            '<' => {
                let token_type = if self.match_next('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token_type);
            }
            '>' => {
                let token_type = if self.match_next('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token_type);
            }
            '/' => {
                if self.match_next('/') {
                    // Go until end of the commented line
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => {
                self.line += 1;
            }
            '"' => self.string(),
            '0'..='9' => self.number(),
            c if is_alpha(c) => self.identifier(),
            _ => self.error(self.line, "Unexpected character."),
        }
    }

    fn error(&self, line: i32, msg: &str) {
        let reporter = self.error_reporter.as_ref().unwrap();
        let mut reporter = reporter.borrow_mut();
        reporter.error(line, msg);
    }

    fn advance(&mut self) -> char {
        let ch = self.source_chars.get(self.current);
        self.current += 1;

        *ch.expect("failed to read char!")
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_with_literal(token_type, None);
    }

    // TODO: Any way to perform better here?
    fn source_substring(&self, start: usize, end: usize) -> String {
        self.source_chars.get(start..end).unwrap().iter().collect()
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal_value: Option<Object>) {
        let text = self.source_substring(self.start, self.current);
        let token = Token::new(token_type, &text, literal_value, self.line);
        self.tokens.push(token);
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if let Some(c) = self.source_chars.get(self.current) {
            if c == &expected {
                self.current += 1;
                return true;
            }
        }

        false
    }

    fn peek(&mut self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        *self.source_chars.get(self.current).unwrap_or(&'\0')
    }

    fn peek_next(&mut self) -> char {
        if self.current + 1 >= self.source_chars.len() {
            return '\0';
        }

        *self.source_chars.get(self.current + 1).unwrap_or(&'\0')
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.error(self.line, "Unterminated string.");
            return;
        }

        // The closing "
        self.advance();

        // Skip the quote marks
        let text = self.source_substring(self.start + 1, self.current - 1);
        self.add_token_with_literal(TokenType::StringLiteral, Some(Object::String(text)));
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // Consume '.'
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let text = self.source_substring(self.start, self.current);
        let value = text
            .parse::<f64>()
            .unwrap_or_else(|_| panic!("failed to parse number: {}", text));

        self.add_token_with_literal(TokenType::Number, Some(Object::Number(value)));
    }

    fn identifier(&mut self) {
        while is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let text = self.source_substring(self.start, self.current);
        let token_type = get_keyword(&text).unwrap_or(TokenType::Identifier);
        self.add_token(token_type);
    }
}

fn is_alpha(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_alpha_numeric(c: char) -> bool {
    is_alpha(c) || c.is_ascii_digit()
}

fn get_keyword(text: &str) -> Option<TokenType> {
    match text {
        "and" => Some(TokenType::And),
        "break" => Some(TokenType::Break),
        "class" => Some(TokenType::Class),
        "else" => Some(TokenType::Else),
        "false" => Some(TokenType::False),
        "for" => Some(TokenType::For),
        "fun" => Some(TokenType::Fun),
        "if" => Some(TokenType::If),
        "nil" => Some(TokenType::Nil),
        "or" => Some(TokenType::Or),
        "print" => Some(TokenType::Print),
        "return" => Some(TokenType::Return),
        "super" => Some(TokenType::Super),
        "this" => Some(TokenType::This),
        "true" => Some(TokenType::True),
        "var" => Some(TokenType::Var),
        "while" => Some(TokenType::While),
        _ => None,
    }
}
