use std::error::Error;
use std::fmt::Display;

use crate::object::Object;
use crate::token::Token;

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug)]
pub struct ResolverError {
    pub token: Option<Token>,
    pub msg: String,
}

impl ResolverError {
    pub fn new<T>(token: Option<Token>, msg: impl AsRef<str>) -> Result<T, Self> {
        Err(Self { token, msg: msg.as_ref().to_owned() })
    }
}

impl Display for ResolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.token {
            Some(token) => write!(f, "[line {}] {}", token.line, self.msg),
            None => write!(f, "[line ?] {}", self.msg),
        }
    }
}

impl Error for ResolverError {}
