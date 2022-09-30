use crate::token::{LiteralValue, Token};

pub trait Visitor<I> {
    type Result;
    type Error;
    fn visit(&mut self, input: &I) -> Result<Self::Result, Self::Error>;
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
    Variable {
        name: Token,
    },
    Assignment {
        name: Token,
        value: Box<Expr>,
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

pub struct AstPrinter;

impl Visitor<Expr> for AstPrinter {
    type Result = String;
    type Error = ();

    fn visit(&mut self, expr: &Expr) -> Result<String, ()> {
        let s = match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => format!(
                "({} {} {})",
                operator.lexeme,
                self.visit(left)?,
                self.visit(right)?
            ),
            Expr::Grouping { expr } => format!("(group {})", self.visit(expr)?),
            Expr::Literal { value } => format!("{}", value),
            Expr::Unary { operator, right } => {
                format!("({} {})", operator.lexeme, self.visit(right)?)
            }
            Expr::Variable { name } => format!("{}", name),
            Expr::Assignment { name, value } => format!("{} = {}", name, self.visit(value)?),
        };
        Ok(s)
    }
}

#[derive(Debug)]
pub enum Stmt {
    Print {
        expr: Expr,
    },
    Expression {
        expr: Expr,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    Block {
        statements: Vec<Stmt>,
    },
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

        let mut printer = AstPrinter;
        let res = printer.visit(&expr);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res, "(* (- 123) (group 45.67))".to_owned());
    }
}
