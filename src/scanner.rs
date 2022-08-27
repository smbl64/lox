use crate::prelude::*;

#[derive(Debug)]
pub struct Scanner {
    input: String,
}

impl Scanner {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_owned(),
        }
    }

    pub fn scan_tokens(&self) -> Vec<Token> {
        todo!()
    }
}
