use crate::prelude::*;

pub struct Interpreter;

pub enum RuntimeError {
    InvalidOperand { operator: Token, msg: &'static str },
}

type InterpreterResult = Result<LiteralValue, RuntimeError>;

impl Visitor<LiteralValue, RuntimeError> for Interpreter {
    fn visit(&self, expr: &Expr) -> InterpreterResult {
        match expr {
            Expr::Literal { value } => Ok(value.clone()),
            Expr::Grouping { expr: inner } => self.visit(inner),
            Expr::Unary { operator, right } => self.visit_unary(operator, right),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.visit_binary(left, operator, right),
        }
    }
}

impl Interpreter {
    fn is_truthy(&self, value: &LiteralValue) -> bool {
        match value {
            LiteralValue::Null => false,
            LiteralValue::Boolean(false) => false,
            _ => true,
        }
    }

    fn visit_unary(&self, operator: &Token, right: &Expr) -> InterpreterResult {
        let value = self.visit(right)?;
        match operator.token_type {
            TokenType::Minus => {
                if let LiteralValue::Number(n) = value {
                    Ok(LiteralValue::Number(-n))
                } else {
                    Err(RuntimeError::InvalidOperand {
                        operator: operator.clone(),
                        msg: "Operand must be a number",
                    })
                }
            }
            TokenType::Bang => Ok(LiteralValue::Boolean(!self.is_truthy(&value))),

            // Unreachable code. We don't have any unary expression except the ones above.
            _ => Ok(LiteralValue::Null),
        }
    }

    fn visit_binary(&self, left: &Expr, operator: &Token, right: &Expr) -> InterpreterResult {
        let left_value = self.visit(left)?;
        let right_value = self.visit(right)?;

        match operator.token_type {
            TokenType::Plus => {
                if let (Some(l), Some(r)) = (left_value.number(), right_value.number()) {
                    Ok(LiteralValue::Number(l + r))
                } else if let (Some(l), Some(r)) = (left_value.string(), right_value.string()) {
                    Ok(LiteralValue::String(format!("{}{}", l, r)))
                } else {
                    Err(RuntimeError::InvalidOperand {
                        operator: operator.clone(),
                        msg: "Operands must be two numbers or two strings",
                    })
                }
            }
            TokenType::Minus => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| LiteralValue::Number(l - r)),
            TokenType::Star => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| LiteralValue::Number(l * r)),
            TokenType::Slash => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| LiteralValue::Number(l / r)),
            TokenType::Greater => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| LiteralValue::Boolean(l > r)),
            TokenType::GreaterEqual => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| LiteralValue::Boolean(l >= r)),
            TokenType::Less => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| LiteralValue::Boolean(l < r)),
            TokenType::LessEqual => self
                .check_number_operands(operator, &left_value, &right_value)
                .map(|(l, r)| LiteralValue::Boolean(l <= r)),

            TokenType::EqualEqual => Ok(LiteralValue::Boolean(left_value == right_value)),
            TokenType::BangEqual => Ok(LiteralValue::Boolean(left_value != right_value)),

            // Unreachable code
            _ => Ok(LiteralValue::Null),
        }
    }

    fn check_number_operands(
        &self,
        operator: &Token,
        left: &LiteralValue,
        right: &LiteralValue,
    ) -> Result<(f64, f64), RuntimeError> {
        if let (Some(l), Some(r)) = (left.number(), right.number()) {
            Ok((l, r))
        } else {
            Err(RuntimeError::InvalidOperand {
                operator: operator.clone(),
                msg: "Operands must be numbers",
            })
        }
    }
}
