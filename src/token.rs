use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::prelude::{Interpreter, RuntimeError};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    StringLiteral,
    Number,

    // Keywords.
    And,
    Break,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EOF,
}

pub trait Callable: Debug {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpret: &mut Interpreter,
        arguments: Vec<Object>,
    ) -> Result<Object, RuntimeError>;
}

#[derive(Debug, Clone)]
pub enum Object {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Callable(Rc<dyn Callable>),
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Boolean(left), Self::Boolean(right)) => left == right,
            (Self::Number(left), Self::Number(right)) => left == right,
            (Self::String(left), Self::String(right)) => left == right,
            (Self::Callable(_), Self::Callable(_)) => false,
            _ => false,
        }
    }
}

impl Eq for Object {}

impl Object {
    pub fn number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }
    pub fn boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }
    pub fn string(&self) -> Option<String> {
        match self {
            Self::String(s) => Some(s.clone()),
            _ => None,
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Boolean(b) => write!(f, "{}", b),
            Self::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Self::String(s) => write!(f, "{}", s),
            Self::Null => write!(f, "nil"),
            Self::Callable(c) => write!(f, "{:?}", c),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Object>,
    pub line: i32,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: &str, literal: Option<Object>, line: i32) -> Self {
        Self {
            token_type,
            lexeme: lexeme.to_owned(),
            literal,
            line,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {} {:?}",
            self.token_type, self.lexeme, self.literal
        )
    }
}
