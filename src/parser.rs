use std::rc::Rc;

use crate::prelude::*;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    errors: Vec<ParserError>,
}

#[derive(Debug)]
pub struct ParserError {
    pub message: String,
    pub token: Token,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0, errors: Vec::new() }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Vec<ParserError>> {
        let mut statements = vec![];
        while !self.is_at_end() {
            // We won't stop if the return value is None. Error reporter
            // keeps track of any error that happens and Lox won't go to next stages.
            // Also, we have the `synchronize` mechanism which helps us to get out of
            // errornous state in statements and move to the next one.
            if let Some(s) = self.declaration() {
                statements.push(s);
            }
        }

        if !self.errors.is_empty() { Err(std::mem::take(&mut self.errors)) } else { Ok(statements) }
    }

    fn declaration(&mut self) -> Option<Stmt> {
        let result = if self.match_tt(&[TokenType::Var]) {
            self.var_declaration()
        } else if self.match_tt(&[TokenType::Class]) {
            self.class()
        } else if self.match_tt(&[TokenType::Fun]) {
            self.function("function")
        } else {
            self.statement()
        };

        if result.is_none() {
            self.synchronize();
            return None;
        }

        result
    }

    fn var_declaration(&mut self) -> Option<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;

        let initializer =
            if self.match_tt(&[TokenType::Equal]) { Some(self.expression()?) } else { None };

        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration.")?;

        Some(Stmt::Var { name, initializer })
    }

    fn class(&mut self) -> Option<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expect class name")?;
        let superclass = if self.match_tt(&[TokenType::Less]) {
            let super_class = self.consume(TokenType::Identifier, "Expect superclass name.")?;
            Some(Expr::Variable { name: super_class })
        } else {
            None
        };

        self.consume(TokenType::LeftBrace, "Expect '{' before class body.")?;
        let mut methods = vec![];
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            methods.push(self.function("method")?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after class body.")?;

        Some(Stmt::Class { name, methods, superclass })
    }

    fn function(&mut self, kind: &str) -> Option<Stmt> {
        let name = self.consume(TokenType::Identifier, format!("Expect {kind} name.").as_str())?;

        self.consume(TokenType::LeftParen, format!("Expect '(' after {kind} name.").as_str())?;

        let mut parameters = vec![];
        if !self.check(&TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    self.error(self.peek().clone(), "Can't have more than 255 parameters.");
                }

                parameters.push(self.consume(TokenType::Identifier, "Expect parameter name.")?);
                if !self.match_tt(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;
        self.consume(TokenType::LeftBrace, format!("Expect '{{' before {kind} body.").as_str())?;

        let body = self.block()?.into_iter().map(Rc::new).collect::<Vec<_>>();

        Some(Stmt::Function { name, params: parameters, body })
    }

    fn statement(&mut self) -> Option<Stmt> {
        if self.match_tt(&[TokenType::If]) {
            self.if_statement()
        } else if self.match_tt(&[TokenType::While]) {
            self.while_statement()
        } else if self.match_tt(&[TokenType::Return]) {
            self.return_statement()
        } else if self.match_tt(&[TokenType::For]) {
            self.for_statement()
        } else if self.match_tt(&[TokenType::Print]) {
            self.print_statement()
        } else if self.match_tt(&[TokenType::Break]) {
            self.break_statement()
        } else if self.match_tt(&[TokenType::LeftBrace]) {
            Some(Stmt::Block { statements: self.block()? })
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Option<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_tt(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Some(Stmt::If { condition, then_branch, else_branch })
    }

    fn return_statement(&mut self) -> Option<Stmt> {
        let keyword = self.previous();
        let value = if self.check(&TokenType::Semicolon) { None } else { Some(self.expression()?) };

        self.consume(TokenType::Semicolon, "Expect ';' after 'return'.")?;
        Some(Stmt::Return { keyword, value })
    }

    fn while_statement(&mut self) -> Option<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after while condition.")?;

        let body = Box::new(self.statement()?);
        Some(Stmt::While { condition, body })
    }

    fn for_statement(&mut self) -> Option<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if self.match_tt(&[TokenType::Semicolon]) {
            None
        } else if self.match_tt(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if self.check(&TokenType::Semicolon) {
            Expr::Literal { value: Literal::Boolean(true) }
        } else {
            self.expression()?
        };
        self.consume(TokenType::Semicolon, "Expect ';' after 'for' condition.")?;

        let increment =
            if self.check(&TokenType::RightParen) { None } else { Some(self.expression()?) };
        self.consume(TokenType::RightParen, "Expect ')' after 'for' clauses.")?;

        let mut body = self.statement()?;

        // Now reconstruct all those parts as a For statement
        if let Some(increment) = increment {
            body = Stmt::Block { statements: vec![body, Stmt::Expression { expr: increment }] };
        }

        body = Stmt::While { condition, body: Box::new(body) };

        if let Some(initializer) = initializer {
            body = Stmt::Block { statements: vec![initializer, body] };
        }

        Some(body)
    }

    fn print_statement(&mut self) -> Option<Stmt> {
        let mut exprs = vec![];
        exprs.push(self.expression()?);
        while self.match_tt(&[TokenType::Comma]) {
            exprs.push(self.expression()?);
        }

        self.consume(TokenType::Semicolon, "Expect ';' after the print statement.")?;
        Some(Stmt::Print { exprs })
    }

    fn break_statement(&mut self) -> Option<Stmt> {
        let token = self.previous();
        self.consume(TokenType::Semicolon, "Expect ';' after 'break'.")?;
        Some(Stmt::Break { token })
    }

    fn block(&mut self) -> Option<Vec<Stmt>> {
        let mut statements = vec![];

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Some(statements)
    }

    fn expression_statement(&mut self) -> Option<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Some(Stmt::Expression { expr })
    }

    fn expression(&mut self) -> Option<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Option<Expr> {
        let expr = self.or()?;

        if self.match_tt(&[TokenType::Equal]) {
            let equals = self.previous();
            let value = Box::new(self.assignment()?);

            if let Expr::Variable { name } = expr {
                return Some(Expr::Assignment { name, value });
            } else if let Expr::Get { object, name } = expr {
                return Some(Expr::Set { object, name, value });
            }

            self.error(equals, "Invalid assignment target.");
        }

        Some(expr)
    }

    fn or(&mut self) -> Option<Expr> {
        let mut expr = self.and()?;

        while self.match_tt(&[TokenType::Or]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Logical { left: Box::new(expr), operator, right: Box::new(right) };
        }

        Some(expr)
    }

    fn and(&mut self) -> Option<Expr> {
        let mut expr = self.equality()?;

        while self.match_tt(&[TokenType::And]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Logical { left: Box::new(expr), operator, right: Box::new(right) };
        }

        Some(expr)
    }

    fn equality(&mut self) -> Option<Expr> {
        let mut expr = self.comparison()?;

        while self.match_tt(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator: Token = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary { left: Box::new(expr), operator, right: Box::new(right) };
        }
        Some(expr)
    }

    fn comparison(&mut self) -> Option<Expr> {
        let mut expr = self.term()?;

        while self.match_tt(&[
            TokenType::GreaterEqual,
            TokenType::Greater,
            TokenType::LessEqual,
            TokenType::Less,
        ]) {
            let operator: Token = self.previous();
            let right = self.term()?;
            expr = Expr::Binary { left: Box::new(expr), operator, right: Box::new(right) };
        }
        Some(expr)
    }

    fn term(&mut self) -> Option<Expr> {
        let mut expr = self.factor()?;

        while self.match_tt(&[TokenType::Minus, TokenType::Plus]) {
            let operator: Token = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary { left: Box::new(expr), operator, right: Box::new(right) };
        }
        Some(expr)
    }

    fn factor(&mut self) -> Option<Expr> {
        let mut expr = self.unary()?;

        while self.match_tt(&[TokenType::Slash, TokenType::Star]) {
            let operator: Token = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary { left: Box::new(expr), operator, right: Box::new(right) };
        }
        Some(expr)
    }

    fn unary(&mut self) -> Option<Expr> {
        if self.match_tt(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Some(Expr::Unary { operator, right: Box::new(right) });
        }

        self.call()
    }

    fn call(&mut self) -> Option<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.match_tt(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_tt(&[TokenType::Dot]) {
                let name =
                    self.consume(TokenType::Identifier, "Expect property name after '.'.")?;
                expr = Expr::Get { object: Box::new(expr), name };
            } else {
                break;
            }
        }

        Some(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Option<Expr> {
        let mut arguments = vec![];

        if !self.check(&TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    // Just report the error, but don't return None yet
                    self.error(self.peek().clone(), "Can't have more than 255 arguments.");
                }

                arguments.push(self.expression()?);

                if !self.match_tt(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
        Some(Expr::Call { callee: Box::new(callee), paren, arguments })
    }

    fn primary(&mut self) -> Option<Expr> {
        if self.match_tt(&[TokenType::False]) {
            return Some(Expr::Literal { value: Literal::Boolean(false) });
        }
        if self.match_tt(&[TokenType::True]) {
            return Some(Expr::Literal { value: Literal::Boolean(true) });
        }

        if self.match_tt(&[TokenType::Super]) {
            let keyword = self.previous();
            self.consume(TokenType::Dot, "Expect '.' after 'super'.")?;
            let method = self.consume(TokenType::Identifier, "Expect superclass method name.")?;
            return Some(Expr::Super { keyword, method });
        }

        if self.match_tt(&[TokenType::Nil]) {
            return Some(Expr::Literal { value: Literal::Null });
        }

        if self.match_tt(&[TokenType::Number, TokenType::StringLiteral]) {
            return Some(Expr::Literal {
                value: self.previous().literal.expect("expecting a number or string here."),
            });
        }

        if self.match_tt(&[TokenType::This]) {
            return Some(Expr::This { keyword: self.previous() });
        }

        if self.match_tt(&[TokenType::Identifier]) {
            return Some(Expr::Variable { name: self.previous() });
        }

        if self.match_tt(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Some(Expr::Grouping { expr: Box::new(expr) });
        }

        self.error(self.peek().clone(), "Expect expression.");
        None
    }

    /// Return the next token if its `token_type` matches the given type as
    /// input. Otherwise, print the error message and return `None`.
    fn consume(&mut self, token_type: TokenType, message: &str) -> Option<Token> {
        if self.check(&token_type) {
            return Some(self.advance());
        }

        self.error(self.peek().clone(), message);
        None
    }

    fn error(&mut self, token: Token, message: &str) {
        self.errors.push(ParserError { message: message.to_owned(), token });
    }

    fn match_tt(&mut self, types: &[TokenType]) -> bool {
        for tt in types {
            if self.check(tt) {
                self.advance();
                return true;
            }
        }

        false
    }

    /// Check to see if the next token's type matches the given `token_type`.
    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type == *token_type
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&mut self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn synchronize(&mut self) {
        self.advance();

        // Move and discard tokens until we find a statement boundary
        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }
}
