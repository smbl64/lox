use std::fmt::Display;

use crate::object::Object;

#[derive(Debug, PartialEq)]
pub enum RuntimeInterrupt {
    Error { line: i32, msg: String },
    Break { line: i32 },
    Return { line: i32, value: Object },
}

impl RuntimeInterrupt {
    pub fn error(line: i32, msg: impl AsRef<str>) -> Self {
        Self::Error { line, msg: msg.as_ref().to_owned() }
    }
}

impl Display for RuntimeInterrupt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeInterrupt::Error { line, msg } => {
                write!(f, "[line {}] {msg}", line)
            }
            RuntimeInterrupt::Break { line } => {
                write!(f, "[line {}] Unexpected break statement", line)
            }
            RuntimeInterrupt::Return { line, .. } => {
                write!(f, "[line {}] Unexpected return statement", line)
            }
        }
    }
}
