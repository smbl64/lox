use crate::prelude::*;

pub struct Interpreter;

pub enum RuntimeError {
    InvalidOperand {
        token: Token,
        operand: LiteralValue,
        msg: &'static str,
    },
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
                        token: operator.clone(),
                        operand: value,
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
        // Both operands are numbers
        let left_value = self.visit(left)?;
        let right_value = self.visit(right)?;

        if let (Some(l), Some(r)) = (left_value.number(), right_value.number()) {
            match operator.token_type {
                TokenType::Minus => return Ok(LiteralValue::Number(l - r)),
                TokenType::Plus => return Ok(LiteralValue::Number(l + r)),
                TokenType::Star => return Ok(LiteralValue::Number(l * r)),
                TokenType::Slash => return Ok(LiteralValue::Number(l / r)),
                TokenType::Greater => return Ok(LiteralValue::Boolean(l > r)),
                TokenType::GreaterEqual => return Ok(LiteralValue::Boolean(l >= r)),
                TokenType::Less => return Ok(LiteralValue::Boolean(l < r)),
                TokenType::LessEqual => return Ok(LiteralValue::Boolean(l <= r)),
                _ => {}
            }
        }

        // Both operands are strings
        if let (Some(l), Some(r)) = (left_value.string(), right_value.string()) {
            if operator.token_type == TokenType::Plus {
                return Ok(LiteralValue::String(format!("{}{}", l, r)));
            }
        }

        if operator.token_type == TokenType::EqualEqual {
            return Ok(LiteralValue::Boolean(left_value == right_value));
        }
        if operator.token_type == TokenType::BangEqual {
            return Ok(LiteralValue::Boolean(left_value != right_value));
        }

        // Unreachable code
        Ok(LiteralValue::Null)
    }
}
