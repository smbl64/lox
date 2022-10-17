use std::{hash::Hash, rc::Rc};

use crate::prelude::*;

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
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: Token,
    },
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    Super {
        keyword: Token,
        method: Token,
    },
    This {
        keyword: Token,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Literal {
        value: Object,
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
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
}

impl Hash for Expr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash based on memory location
        self.unique_id().hash(state);
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        self.unique_id() == other.unique_id()
    }
}

impl Eq for Expr {}

impl Expr {
    pub fn int_literal(v: f64) -> Expr {
        Expr::Literal {
            value: Object::Number(v),
        }
    }

    pub fn str_literal(s: &str) -> Expr {
        Expr::Literal {
            value: Object::String(s.to_owned()),
        }
    }

    pub fn unique_id(&self) -> usize {
        std::ptr::addr_of!(*self) as usize
    }
}

#[derive(Debug)]
pub enum Stmt {
    Break {
        token: Token,
    },
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
    Class {
        name: Token,
        methods: Vec<Stmt>,
        superclass: Option<Expr>,
    },
    Print {
        exprs: Vec<Expr>,
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
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Rc<Stmt>>,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}

impl AsRef<Stmt> for Stmt {
    fn as_ref(&self) -> &Stmt {
        self
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
            Expr::Logical {
                left,
                operator,
                right,
            } => format!(
                "({} {} {})",
                operator.lexeme,
                self.visit(left)?,
                self.visit(right)?
            ),
            Expr::Call {
                callee,
                paren: _,
                arguments,
            } => format!("{:?}({:?})", self.visit(callee)?, arguments),
            Expr::This { keyword } => format!("{}", keyword),
            Expr::Get { object, name } => format!("{:?}.{}", self.visit(object)?, name),
            Expr::Set {
                object,
                name,
                value,
            } => format!(
                "{:?}.{} = {:?}",
                self.visit(object)?,
                name,
                self.visit(value)?
            ),
            Expr::Super { keyword, method } => format!("{}.{}", keyword, method),
        };
        Ok(s)
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

        let mut printer = AstPrinter;
        let res = printer.visit(&expr);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res, "(* (- 123) (group 45.67))".to_owned());
    }
}
