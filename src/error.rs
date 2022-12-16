use std::fmt::Display;

use crate::object::Object;

#[derive(Debug, PartialEq)]
pub enum RuntimeInterrupt {
    /// A runtime error has occured.
    Error { line: u32, msg: String },
    /// A break statement has been reached.
    Break { line: u32 },
    /// A return statement has been reached.
    Return { line: u32, value: Object },
}

impl RuntimeInterrupt {
    pub fn error(line: u32, msg: impl AsRef<str>) -> Self {
        Self::Error { line, msg: msg.as_ref().to_owned() }
    }
}

impl Display for RuntimeInterrupt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeInterrupt::Error { line, msg } => {
                write!(f, "[line {line}] {msg}")
            }
            RuntimeInterrupt::Break { line } => {
                write!(f, "[line {line}] Unexpected break statement")
            }
            RuntimeInterrupt::Return { line, .. } => {
                write!(f, "[line {line}] Unexpected return statement")
            }
        }
    }
}
