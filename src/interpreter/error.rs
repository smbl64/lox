use crate::object::Object;
use crate::token::Token;
use std::error::Error;
use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    InvalidOperand { operator: Token, msg: String },
    UndefinedVariable { name: Token, msg: String },
    Break { token: Token },
    Return { token: Token, value: Object },
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::InvalidOperand { operator, msg } => {
                write!(f, "[line {}] {}", operator.line, msg)
            }
            RuntimeError::UndefinedVariable { name, msg } => {
                write!(f, "[line {}] {}", name.line, msg)
            }
            RuntimeError::Break { token } => {
                write!(f, "[line {}] Unexpected break statement", token.line)
            }
            RuntimeError::Return { token, value: _ } => {
                write!(f, "[line {}] Unexpected return statement", token.line)
            }
        }
    }
}

#[derive(Debug)]
pub struct ResolverError {
    pub token: Token,
    pub msg: String,
}

impl Display for ResolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] {}", self.token.line, self.msg)
    }
}

impl Error for ResolverError {}
