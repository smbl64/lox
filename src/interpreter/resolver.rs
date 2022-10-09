use std::error::Error;
use std::{collections::HashMap, fmt::Display};

use crate::{
    prelude::{Expr, Stmt, Visitor},
    token::Token,
};

use super::Interpreter;

#[derive(Debug)]
pub struct ResolverError {
    token: Token,
    msg: String,
}

impl Display for ResolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] {}", self.token.line, self.msg)
    }
}

impl Error for ResolverError {}

#[derive(Debug, Clone, PartialEq, Copy)]
enum FunctionType {
    None,
    Function,
}

/// Resolver uses static analysis to bind local variables to the correct envorinment.
pub struct Resolver<'i> {
    interpreter: &'i mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}

impl<'i> Resolver<'i> {
    pub fn new(interpreter: &'i mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: vec![],
            current_function: FunctionType::None,
        }
    }
}

impl<'a> Visitor<Stmt> for Resolver<'a> {
    type Error = ResolverError;
    type Result = ();

    fn visit(&mut self, input: &Stmt) -> Result<Self::Result, Self::Error> {
        match input {
            Stmt::Block { statements } => {
                self.begin_scope();
                self.resolve(statements)?;
                self.end_scope();

                Ok(())
            }
            Stmt::Var { name, initializer } => {
                // We use a 3 step process, so users can't use the same variable in
                // variable definition: declare -> initialize -> define
                self.declare(name)?;
                if let Some(initializer) = initializer {
                    self.resolve_expr(initializer)?;
                }
                self.define(name);
                Ok(())
            }
            Stmt::Function { name, params, body } => {
                // Unlike variables, we declare and define functions before processing
                // their body. This way, functions can recursively call themselves.
                self.declare(name)?;
                self.define(name);

                let enclosing_func = self.current_function;
                self.current_function = FunctionType::Function;

                // Resolve function's body
                self.begin_scope();
                for param in params {
                    self.declare(param)?;
                    self.define(param);
                }

                self.resolve(body)?;
                self.end_scope();
                self.current_function = enclosing_func;

                Ok(())
            }
            Stmt::Expression { expr } => self.resolve_expr(expr),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition)?;
                self.resolve_single_stmt(&then_branch)?;
                if let Some(stmt) = else_branch {
                    self.resolve_single_stmt(stmt)?;
                }
                Ok(())
            }
            Stmt::Print { exprs } => {
                for ex in exprs {
                    self.resolve_expr(ex)?;
                }
                Ok(())
            }
            Stmt::Return { keyword, value } => {
                if self.current_function == FunctionType::None {
                    return Err(ResolverError {
                        token: keyword.clone(),
                        msg: "Can't return from top-level code".to_owned(),
                    });
                }

                if let Some(expr) = value {
                    self.resolve_expr(expr)?;
                }
                Ok(())
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(condition)?;
                self.resolve_single_stmt(body)
            }
            Stmt::Break { token: _ } => Ok(()),
        }
    }
}

impl<'a> Resolver<'a> {
    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) -> Result<(), ResolverError> {
        if self.scopes.is_empty() {
            return Ok(());
        }

        let last_idx = self.scopes.len() - 1;
        let last = self.scopes.get_mut(last_idx).unwrap();

        if last.contains_key(&name.lexeme) {
            return Err(ResolverError {
                token: name.clone(),
                msg: "Already a variable with this name in this scope.".to_owned(),
            });
        }

        last.insert(name.lexeme.clone(), false);
        Ok(())
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let last_idx = self.scopes.len() - 1;
        let last = self.scopes.get_mut(last_idx).unwrap();
        last.insert(name.lexeme.clone(), true);
    }

    pub fn resolve<I, R>(&mut self, statements: I) -> Result<(), ResolverError>
    where
        I: IntoIterator<Item = R>,
        R: AsRef<Stmt>,
    {
        for stmt in statements {
            self.resolve_single_stmt(stmt.as_ref())?;
        }

        Ok(())
    }

    fn resolve_single_stmt(&mut self, stmt: &Stmt) -> Result<(), ResolverError> {
        self.visit(stmt)
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<(), ResolverError> {
        self.visit(expr)
    }
}

impl<'a> Visitor<Expr> for Resolver<'a> {
    type Error = ResolverError;
    type Result = ();

    fn visit(&mut self, input: &Expr) -> Result<Self::Result, Self::Error> {
        match input {
            Expr::Variable { name } => {
                if !self.scopes.is_empty() {
                    let last_idx = self.scopes.len() - 1;
                    let scope = self.scopes.get(last_idx).unwrap();

                    if let Some(false) = scope.get(&name.lexeme) {
                        return Err(ResolverError {
                            token: name.clone(),
                            msg: "Can't read local variable in its own initialization".to_owned(),
                        });
                    }
                }

                self.resolve_local(input, name)?;
                Ok(())
            }
            Expr::Assignment { name, value } => {
                self.resolve_expr(&value)?;
                self.resolve_local(input, name)
            }
            Expr::Binary {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)
            }
            Expr::Call {
                callee,
                paren: _,
                arguments,
            } => {
                self.resolve_expr(&callee)?;
                for arg in arguments {
                    self.resolve_expr(&arg)?;
                }
                Ok(())
            }
            Expr::Grouping { expr } => self.resolve_expr(expr),
            Expr::Literal { value: _ } => Ok(()),
            Expr::Unary { operator: _, right } => self.resolve_expr(right),
            Expr::Logical {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)
            }
        }
    }
}

impl<'a> Resolver<'a> {
    fn resolve_local(&mut self, input: &Expr, name: &Token) -> Result<(), ResolverError> {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(input, self.scopes.len() - i - 1);
                return Ok(());
            }
        }

        Ok(())
    }
}
