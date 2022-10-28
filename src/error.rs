use std::fmt::Display;

use crate::object::Object;
use crate::token::Token;

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    InvalidOperand { operator: Token, msg: String },
    UndefinedVariable { name: Token, msg: String },
    Generic { name: Token, msg: String },
    Break { token: Token },
    Return { token: Token, value: Object },
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::InvalidOperand { operator, msg } => {
                write!(f, "[line {}] {msg}", operator.line)
            }
            RuntimeError::UndefinedVariable { name, msg } => {
                write!(f, "[line {}] {msg}", name.line)
            }
            RuntimeError::Generic { name, msg } => {
                write!(f, "[line {}] {msg}", name.line)
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
