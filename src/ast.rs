use std::hash::Hash;
use std::rc::Rc;

use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UniqueId(pub usize);

#[derive(Debug)]
pub enum Expr {
    Binary { left: Box<Expr>, operator: Token, right: Box<Expr> },
    Call { callee: Box<Expr>, paren: Token, arguments: Vec<Expr> },
    Get { object: Box<Expr>, name: Token },
    Set { object: Box<Expr>, name: Token, value: Box<Expr> },
    Super { keyword: Token, method: Token },
    This { keyword: Token },
    Grouping { expr: Box<Expr> },
    Literal { value: Literal },
    Unary { operator: Token, right: Box<Expr> },
    Variable { name: Token },
    Assignment { name: Token, value: Box<Expr> },
    Logical { left: Box<Expr>, operator: Token, right: Box<Expr> },
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
        Expr::Literal { value: Literal::Number(v) }
    }

    pub fn str_literal(s: &str) -> Expr {
        Expr::Literal { value: Literal::String(s.to_owned()) }
    }

    pub fn unique_id(&self) -> UniqueId {
        UniqueId(std::ptr::addr_of!(*self) as usize)
    }
}

#[derive(Debug)]
pub enum Stmt {
    Break { token: Token },
    Return { keyword: Token, value: Option<Expr> },
    Class { name: Token, methods: Vec<Stmt>, superclass: Option<Expr> },
    Print { exprs: Vec<Expr> },
    Expression { expr: Expr },
    Var { name: Token, initializer: Option<Expr> },
    Block { statements: Vec<Stmt> },
    Function { name: Token, params: Vec<Token>, body: Vec<Rc<Stmt>> },
    If { condition: Expr, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>> },
    While { condition: Expr, body: Box<Stmt> },
}

impl AsRef<Stmt> for Stmt {
    fn as_ref(&self) -> &Stmt {
        self
    }
}

pub struct AstPrinter;

impl AstPrinter {
    pub fn to_string(expr: &Expr) -> String {
        match expr {
            Expr::Binary { left, operator, right } => {
                format!(
                    "({} {} {})",
                    operator.lexeme,
                    Self::to_string(left),
                    Self::to_string(right)
                )
            }
            Expr::Grouping { expr } => format!("(group {})", Self::to_string(expr)),
            Expr::Literal { value } => format!("{value}"),
            Expr::Unary { operator, right } => {
                format!("({} {})", operator.lexeme, Self::to_string(right))
            }
            Expr::Variable { name } => format!("{name}"),
            Expr::Assignment { name, value } => format!("{name} = {}", Self::to_string(value)),
            Expr::Logical { left, operator, right } => {
                format!(
                    "({} {} {})",
                    operator.lexeme,
                    Self::to_string(left),
                    Self::to_string(right)
                )
            }
            Expr::Call { callee, paren: _, arguments } => {
                format!("{:?}({arguments:?})", Self::to_string(callee))
            }
            Expr::This { keyword } => format!("{keyword}"),
            Expr::Get { object, name } => format!("{:?}.{name}", Self::to_string(object)),
            Expr::Set { object, name, value } => {
                format!("{:?}.{name} = {:?}", Self::to_string(object), Self::to_string(value))
            }
            Expr::Super { keyword, method } => format!("{keyword}.{method}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenType;

    #[test]
    fn print_an_ast() {
        // This is '-123 * (45.67)'
        let expr = Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: Token::new(TokenType::Minus, "-", None, 1),
                right: Box::new(Expr::int_literal(123.0)),
            }),
            operator: Token::new(TokenType::Star, "*", None, 1),
            right: Box::new(Expr::Grouping { expr: Box::new(Expr::int_literal(45.67)) }),
        };

        let res = AstPrinter::to_string(&expr);
        assert_eq!(res, "(* (- 123) (group 45.67))".to_owned());
    }
}
