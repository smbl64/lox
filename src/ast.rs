use crate::{
    report,
    token::{LiteralValue, Token, TokenType},
};

trait Visitor<R> {
    fn visit(&self, expr: &Expr) -> R;
}

#[derive(Debug)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Literal {
        value: LiteralValue,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

impl Expr {
    pub fn int_literal(v: f64) -> Expr {
        Expr::Literal {
            value: LiteralValue::Number(v),
        }
    }

    pub fn str_literal(s: &str) -> Expr {
        Expr::Literal {
            value: LiteralValue::String(s.to_owned()),
        }
    }
}

struct AstPrinter;

impl Visitor<String> for AstPrinter {
    fn visit(&self, expr: &Expr) -> String {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => format!(
                "({} {} {})",
                operator.lexeme,
                self.visit(left),
                self.visit(right)
            ),
            Expr::Grouping { expr } => format!("(group {})", self.visit(expr)),
            Expr::Literal { value } => format!("{}", value),
            Expr::Unary { operator, right } => {
                format!("({} {})", operator.lexeme, self.visit(right))
            }
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while self.match_tt(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator: Token = self.previous();
            let right = self.comparison();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        expr
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();

        while self.match_tt(&[
            TokenType::GreaterEqual,
            TokenType::Greater,
            TokenType::LessEqual,
            TokenType::Less,
        ]) {
            let operator: Token = self.previous();
            let right = self.term();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        while self.match_tt(&[TokenType::Minus, TokenType::Plus]) {
            let operator: Token = self.previous();
            let right = self.factor();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.match_tt(&[TokenType::Slash, TokenType::Star]) {
            let operator: Token = self.previous();
            let right = self.unary();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        expr
    }

    fn unary(&mut self) -> Expr {
        if self.match_tt(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary();
            return Expr::Unary {
                operator,
                right: Box::new(right),
            };
        }

        self.primary()
    }

    fn primary(&mut self) -> Expr {
        if self.match_tt(&[TokenType::False]) {
            return Expr::Literal {
                value: LiteralValue::Boolean(false),
            };
        }
        if self.match_tt(&[TokenType::True]) {
            return Expr::Literal {
                value: LiteralValue::Boolean(true),
            };
        }
        if self.match_tt(&[TokenType::Nil]) {
            return Expr::Literal {
                value: LiteralValue::Null,
            };
        }
        if self.match_tt(&[TokenType::Number, TokenType::StringLiteral]) {
            return Expr::Literal {
                value: self
                    .previous()
                    .literal
                    .expect("expecting a number or string here"),
            };
        }

        if self.match_tt(&[TokenType::LeftParen]) {
            let expr = self.expression();
            self.consume(TokenType::RightParen, "Expect ')' after expression.");
            return Expr::Grouping {
                expr: Box::new(expr),
            };
        }

        // TODO
        todo!()
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Token {
        if self.check(&token_type) {
            return self.advance();
        }

        self.error(self.peek(), message);
        panic!();
    }

    fn error(&self, token: Token, message: &str) {
        if token.token_type == TokenType::EOF {
            report(token.line, " at end", message);
        } else {
            report(token.line, &format!(" at '{}'", token.lexeme), message);
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
}

#[cfg(test)]
mod tests {
    use crate::token::TokenType;

    use super::*;

    #[test]
    fn print_an_ast() {
        // This is '-123 * (45.67)'
        let expr = Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: Token::new(TokenType::Minus, "-", None, 1),
                right: Box::new(Expr::int_literal(123.0)),
            }),
            operator: Token::new(TokenType::Star, "*", None, 1),
            right: Box::new(Expr::Grouping {
                expr: Box::new(Expr::int_literal(45.67)),
            }),
        };

        let printer = AstPrinter;
        let res = printer.visit(&expr);
        assert_eq!(res, "(* (- 123) (group 45.67))".to_owned());
    }
}
