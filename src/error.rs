use std::fmt::Display;

use crate::object::Object;

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    Generic { line: i32, msg: String },
    Break { line: i32 },
    Return { line: i32, value: Object },
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::Generic { line, msg } => {
                write!(f, "[line {}] {msg}", line)
            }
            RuntimeError::Break { line } => {
                write!(f, "[line {}] Unexpected break statement", line)
            }
            RuntimeError::Return { line, .. } => {
                write!(f, "[line {}] Unexpected return statement", line)
            }
        }
    }
}
