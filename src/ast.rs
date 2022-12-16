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
