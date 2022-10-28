use super::InterpreterResult;
use crate::prelude::*;

impl Interpreter {
    pub fn evaluate_expr(&mut self, expr: &Expr) -> InterpreterResult {
        match expr {
            Expr::Literal { value } => Ok(value.clone().into()),
            Expr::Grouping { expr: inner } => self.evaluate_expr(inner.as_ref()),
            Expr::Unary { operator, right } => self.evaluate_unary(operator, right),
            Expr::Binary { left, operator, right } => self.evaluate_binary(left, operator, right),
            Expr::Variable { name } => self.lookup_variable(name, expr),
            Expr::Assignment { name, value } => {
                let value = self.evaluate_expr(value.as_ref())?;

                if let Some(&distance) = self.locals.get(&expr.unique_id()) {
                    self.environment.borrow_mut().assign_at(distance, name, value.clone())?;
                } else {
                    self.globals.borrow_mut().assign(name, value.clone())?;
                }

                Ok(value)
            }
            Expr::Get { object, name } => {
                let object = self.evaluate_expr(object)?;
                if let Object::Instance(ref instance) = object {
                    instance.borrow().get(name, &object)
                } else {
                    Err(RuntimeError::generic(name.line, "Only instances have properties"))
                }
            }
            Expr::Set { object, name, value } => {
                let value = self.evaluate_expr(value)?;
                let object = self.evaluate_expr(object)?;

                if let Object::Instance(instance) = object {
                    instance.borrow_mut().set(name, value.clone());
                    Ok(value)
                } else {
                    Err(RuntimeError::generic(name.line, "Only instances have properties"))
                }
            }
            Expr::Super { keyword, method: method_name } => {
                let distance = *self.locals.get(&expr.unique_id()).expect("Cannot find distance");

                let superclass = self.environment.borrow().get_at(distance, keyword)?;
                let superclass = match superclass {
                    Object::Class(c) => c,
                    _ => panic!("Superclass is not wrapped in Object::Class"),
                };

                let this = Token::new(TokenType::Identifier, "this", None, -1);
                let instance = self.environment.borrow().get_at(distance - 1, &this)?;

                let method = superclass.borrow().find_method(&method_name.lexeme);

                if let Some(method) = method {
                    Ok(Object::Callable(method.bind(instance)))
                } else {
                    Err(RuntimeError::generic(
                        method_name.line,
                        format!("Undefined property '{}'", method_name.lexeme),
                    ))
                }
            }
            Expr::This { keyword } => self.lookup_variable(keyword, expr),
            Expr::Logical { left, operator, right } => {
                let left_val = self.evaluate_expr(left)?;

                if operator.token_type == TokenType::Or {
                    if self.is_truthy(&left_val) {
                        return Ok(left_val);
                    }
                } else {
                    // TokenType::And
                    if !self.is_truthy(&left_val) {
                        return Ok(left_val);
                    }
                }

                self.evaluate_expr(right)
            }
            Expr::Call { callee, paren, arguments } => {
                let callee = self.evaluate_expr(callee)?;
                match callee {
                    Object::Callable(callable) => {
                        if callable.arity() != arguments.len() {
                            return Err(RuntimeError::generic(
                                paren.line,
                                format!(
                                    "Expected {} arguments, got {}",
                                    callable.arity(),
                                    arguments.len()
                                ),
                            ));
                        }
                        // Evaluate all arguments
                        let mut args = vec![];
                        for arg in arguments {
                            args.push(self.evaluate_expr(arg)?);
                        }

                        callable.call(self, args)
                    }
                    Object::Class(class) => {
                        let arity = class.borrow().arity();
                        if arity != arguments.len() {
                            return Err(RuntimeError::generic(
                                paren.line,
                                format!("Expected {} arguments, got {}", arity, arguments.len()),
                            ));
                        }

                        // Evaluate all arguments
                        let mut args = vec![];
                        for arg in arguments {
                            args.push(self.evaluate_expr(arg)?);
                        }

                        Class::construct(class, args, self).map(Object::Instance)
                    }
                    _ => Err(RuntimeError::generic(
                        paren.line,
                        "Can only call functions and classes",
                    )),
                }
            }
        }
    }

    pub(super) fn is_truthy(&self, value: &Object) -> bool {
        !matches!(value, Object::Null | Object::Boolean(false))
    }

    fn evaluate_unary(&mut self, operator: &Token, right: &Expr) -> InterpreterResult {
        let value = self.evaluate_expr(right)?;
        match operator.token_type {
            TokenType::Minus => {
                if let Object::Number(n) = value {
                    Ok(Object::Number(-n))
                } else {
                    Err(RuntimeError::generic(operator.line, "Operand must be a number"))
                }
            }
            TokenType::Bang => Ok(Object::Boolean(!self.is_truthy(&value))),

            // Unreachable code. We don't have any unary expression except the ones above.
            _ => Ok(Object::Null),
        }
    }

    fn evaluate_binary(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> InterpreterResult {
        let left_value = self.evaluate_expr(left)?;
        let right_value = self.evaluate_expr(right)?;

        match operator.token_type {
            TokenType::Plus => {
                if let (Some(l), Some(r)) = (left_value.number(), right_value.number()) {
                    Ok(Object::Number(l + r))
                } else if let (Some(l), Some(r)) = (left_value.string(), right_value.string()) {
                    Ok(Object::String(format!("{l}{r}")))
                } else {
                    Err(RuntimeError::generic(
                        operator.line,
                        "Operands must be two numbers or two strings",
                    ))
                }
            }
            TokenType::Minus => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| Object::Number(l - r)),
            TokenType::Star => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| Object::Number(l * r)),
            TokenType::Slash => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| Object::Number(l / r)),
            TokenType::Greater => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| Object::Boolean(l > r)),
            TokenType::GreaterEqual => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| Object::Boolean(l >= r)),
            TokenType::Less => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| Object::Boolean(l < r)),
            TokenType::LessEqual => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| Object::Boolean(l <= r)),

            TokenType::EqualEqual => Ok(Object::Boolean(left_value == right_value)),
            TokenType::BangEqual => Ok(Object::Boolean(left_value != right_value)),

            // Unreachable code
            _ => Ok(Object::Null),
        }
    }

    fn check_number_operands(
        &self,
        operator: &Token,
        left: &Object,
        right: &Object,
    ) -> Result<(f64, f64), RuntimeError> {
        if let (Some(l), Some(r)) = (left.number(), right.number()) {
            Ok((l, r))
        } else {
            Err(RuntimeError::generic(operator.line, "Operands must be numbers"))
        }
    }

    fn lookup_variable(&self, name: &Token, expr: &Expr) -> Result<Object, RuntimeError> {
        if let Some(&distance) = self.locals.get(&expr.unique_id()) {
            self.environment.borrow().get_at(distance, name)
        } else {
            self.globals.borrow().get(name)
        }
    }
}
