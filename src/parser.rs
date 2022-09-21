use crate::prelude::*;
use crate::report;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Option<Vec<Stmt>> {
        let mut statements = vec![];
        while !self.is_at_end() {
            statements.push(self.statement()?);
        }

        Some(statements)
    }

    fn statement(&mut self) -> Option<Stmt> {
        if self.match_tt(&[TokenType::Print]) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Option<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression");
        Some(Stmt::Print { expr })
    }

    fn expression_statement(&mut self) -> Option<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression");
        Some(Stmt::Expression { expr })
    }

    fn expression(&mut self) -> Option<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Option<Expr> {
        let mut expr = self.comparison()?;

        while self.match_tt(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator: Token = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Some(expr)
    }

    fn comparison(&mut self) -> Option<Expr> {
        let mut expr = self.term()?;

        while self.match_tt(&[
            TokenType::GreaterEqual,
            TokenType::Greater,
            TokenType::LessEqual,
            TokenType::Less,
        ]) {
            let operator: Token = self.previous();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Some(expr)
    }

    fn term(&mut self) -> Option<Expr> {
        let mut expr = self.factor()?;

        while self.match_tt(&[TokenType::Minus, TokenType::Plus]) {
            let operator: Token = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Some(expr)
    }

    fn factor(&mut self) -> Option<Expr> {
        let mut expr = self.unary()?;

        while self.match_tt(&[TokenType::Slash, TokenType::Star]) {
            let operator: Token = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Some(expr)
    }

    fn unary(&mut self) -> Option<Expr> {
        if self.match_tt(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Some(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Option<Expr> {
        if self.match_tt(&[TokenType::False]) {
            return Some(Expr::Literal {
                value: LiteralValue::Boolean(false),
            });
        }
        if self.match_tt(&[TokenType::True]) {
            return Some(Expr::Literal {
                value: LiteralValue::Boolean(true),
            });
        }
        if self.match_tt(&[TokenType::Nil]) {
            return Some(Expr::Literal {
                value: LiteralValue::Null,
            });
        }
        if self.match_tt(&[TokenType::Number, TokenType::StringLiteral]) {
            return Some(Expr::Literal {
                value: self
                    .previous()
                    .literal
                    .expect("expecting a number or string here"),
            });
        }
        if self.match_tt(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Some(Expr::Grouping {
                expr: Box::new(expr),
            });
        }

        self.error(self.peek(), "Expect expression.");
        None
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Option<Token> {
        if self.check(&token_type) {
            return Some(self.advance());
        }

        self.error(self.peek(), message);
        None
    }

    fn error(&self, token: Token, message: &str) {
        if token.token_type == TokenType::EOF {
            report(token.line, "at end", message);
        } else {
            report(token.line, &format!("at '{}'", token.lexeme), message);
        }
    }

    fn match_tt(&mut self, types: &[TokenType]) -> bool {
        for tt in types {
            if self.check(tt) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&self, tt: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type == *tt
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.current).unwrap().clone()
    }

    fn previous(&mut self) -> Token {
        self.tokens.get(self.current - 1).unwrap().clone()
    }

    fn synchronize(&mut self) {
        self.advance();

        // Move and discard tokens until we find a statement boundary
        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }
}
