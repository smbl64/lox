use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;

use super::Interpreter;
use crate::prelude::{Expr, Stmt};
use crate::token::Token;

#[derive(Debug, Clone, PartialEq, Copy)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Debug, Clone, PartialEq, Copy)]
enum ClassType {
    None,
    Class,
    SubClass,
}

/// Resolver uses static analysis to bind local variables to the correct
/// envorinment.
pub struct Resolver<'i> {
    interpreter: &'i mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl<'i> Resolver<'i> {
    pub fn new(interpreter: &'i mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: vec![],
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }
}

impl<'a> Resolver<'a> {
    fn visit_stmt(&mut self, input: &Stmt) -> Result<(), ResolverError> {
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
            Stmt::Class { name, methods, superclass } => {
                let enclosing_class = self.current_class;
                self.current_class = ClassType::Class;

                self.declare(name)?;
                self.define(name);

                if let Some(superclass) = superclass {
                    // Make sure super class has a different name!
                    if let Expr::Variable { name: super_name } = superclass {
                        if super_name.lexeme == name.lexeme {
                            return ResolverError::new(
                                Some(super_name.clone()),
                                "A class can't inherit from itself.",
                            );
                        }
                    } else {
                        panic!("Superclass is not enclosed in a Expr::Variable!");
                    }

                    self.current_class = ClassType::SubClass;
                    self.resolve_expr(superclass)?;
                }

                if superclass.is_some() {
                    self.begin_scope();
                    // Safe to unwrap, because we're calling begin_scope before it
                    self.peek_mut_scope().unwrap().insert("super".to_owned(), true);
                }

                self.begin_scope();
                // Safe to unwrap, because we're calling begin_scope before it
                self.peek_mut_scope().unwrap().insert("this".to_owned(), true);

                for method in methods {
                    let is_initializer = match method {
                        Stmt::Function { name, params: _, body: _ } => name.lexeme == "init",
                        _ => {
                            // This should not happen if the parser
                            // does its job properly!
                            return ResolverError::new(
                                None,
                                "Method must be a function statement.",
                            );
                        }
                    };

                    let func_type = if is_initializer {
                        FunctionType::Initializer
                    } else {
                        FunctionType::Method
                    };

                    self.resolve_function(method, func_type)?;
                }

                self.end_scope();

                if superclass.is_some() {
                    self.end_scope();
                }

                self.current_class = enclosing_class;
                Ok(())
            }
            Stmt::Function { name, params: _, body: _ } => {
                // Unlike variables, we declare and define functions before processing
                // their body. This way, functions can recursively call themselves.
                self.declare(name)?;
                self.define(name);

                self.resolve_function(input, FunctionType::Function)
            }
            Stmt::Expression { expr } => self.resolve_expr(expr),
            Stmt::If { condition, then_branch, else_branch } => {
                self.resolve_expr(condition)?;
                self.resolve_single_stmt(then_branch)?;
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
                    return ResolverError::new(
                        Some(keyword.clone()),
                        "Can't return from top-level code.",
                    );
                }

                if let Some(expr) = value {
                    // Cannot return anything from "init" function
                    if self.current_function == FunctionType::Initializer {
                        return ResolverError::new(
                            Some(keyword.clone()),
                            "Can't return a value from an initializer.",
                        );
                    }
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
            return ResolverError::new(
                Some(name.clone()),
                "Already a variable with this name in this scope.",
            );
        }

        last.insert(name.lexeme.clone(), false);
        Ok(())
    }

    fn define(&mut self, name: &Token) {
        // TODO Refactor to use `peek_mut_scope` here and other places
        if self.scopes.is_empty() {
            return;
        }

        let last_idx = self.scopes.len() - 1;
        let last = self.scopes.get_mut(last_idx).unwrap();
        last.insert(name.lexeme.clone(), true);
    }

    fn peek_mut_scope(&mut self) -> Option<&mut HashMap<String, bool>> {
        if self.scopes.is_empty() {
            return None;
        }

        let last_idx = self.scopes.len() - 1;
        Some(self.scopes.get_mut(last_idx).unwrap())
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
        self.visit_stmt(stmt)
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<(), ResolverError> {
        self.visit_expr(expr)
    }

    fn resolve_this(&mut self, expr: &Expr, keyword: &Token) -> Result<(), ResolverError> {
        if self.current_class == ClassType::None {
            return ResolverError::new(
                Some(keyword.clone()),
                "Can't use 'this' outside of a class.",
            );
        }

        self.resolve_local(expr, keyword)
    }

    fn resolve_function(
        &mut self,
        stmt: &Stmt,
        func_type: FunctionType,
    ) -> Result<(), ResolverError> {
        if let Stmt::Function { name: _, params, body } = stmt {
            let enclosing_func = self.current_function;
            self.current_function = func_type;

            self.begin_scope();
            for param in params {
                self.declare(param)?;
                self.define(param);
            }

            self.resolve(body)?;
            self.end_scope();
            self.current_function = enclosing_func;
            Ok(())
        } else {
            ResolverError::new(None, "Expected a function.")
        }
    }
}

impl<'a> Resolver<'a> {
    fn visit_expr(&mut self, input: &Expr) -> Result<(), ResolverError> {
        match input {
            Expr::Variable { name } => {
                if !self.scopes.is_empty() {
                    let last_idx = self.scopes.len() - 1;
                    let scope = self.scopes.get(last_idx).unwrap();

                    if let Some(false) = scope.get(&name.lexeme) {
                        return ResolverError::new(
                            Some(name.clone()),
                            "Can't read local variable in its own initializer.",
                        );
                    }
                }

                self.resolve_local(input, name)
            }
            Expr::Assignment { name, value } => {
                self.resolve_expr(value)?;
                self.resolve_local(input, name)
            }
            Expr::Super { keyword, method: _ } => {
                if self.current_class == ClassType::None {
                    ResolverError::new(
                        Some(keyword.clone()),
                        "Can't use 'super' outside of a class.",
                    )
                } else if self.current_class != ClassType::SubClass {
                    ResolverError::new(
                        Some(keyword.clone()),
                        "Can't use 'super' in a class with no superclass.",
                    )
                } else {
                    self.resolve_local(input, keyword)
                }
            }
            Expr::Binary { left, operator: _, right } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)
            }
            Expr::Call { callee, paren: _, arguments } => {
                self.resolve_expr(callee)?;
                for arg in arguments {
                    self.resolve_expr(arg)?;
                }
                Ok(())
            }
            Expr::Get { object, name: _ } => {
                self.resolve_expr(object)?;
                Ok(())
            }
            Expr::Set { object, name: _, value } => {
                self.resolve_expr(object)?;
                self.resolve_expr(value)?;
                Ok(())
            }
            Expr::This { keyword } => self.resolve_this(input, keyword),
            Expr::Grouping { expr } => self.resolve_expr(expr),
            Expr::Literal { value: _ } => Ok(()),
            Expr::Unary { operator: _, right } => self.resolve_expr(right),
            Expr::Logical { left, operator: _, right } => {
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
            Some(token) => {
                write!(f, "[line {}] Error at '{}': {}", token.line, token.lexeme, self.msg)
            }
            None => write!(f, "{}", self.msg),
        }
    }
}

impl Error for ResolverError {}
