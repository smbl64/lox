use crate::prelude::*;

pub struct Interpreter;

impl Visitor<LiteralValue> for Interpreter {
    fn visit(&self, expr: &Expr) -> LiteralValue {
        match expr {
            Expr::Literal { value } => value.clone(),
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

    fn visit_unary(&self, operator: &Token, right: &Expr) -> LiteralValue {
        let value = self.visit(right);

        if operator.token_type == TokenType::Minus {
            if let LiteralValue::Number(n) = value {
                return LiteralValue::Number(-n);
            }
        } else if operator.token_type == TokenType::Bang {
            return LiteralValue::Boolean(!self.is_truthy(&value));
        }

        // Unreachable code. We don't any unary expression except the ones above.
        LiteralValue::Null
    }

    fn visit_binary(&self, left: &Expr, operator: &Token, right: &Expr) -> LiteralValue {
        // Both operands are numbers
        let left_value = self.visit(left);
        let right_value = self.visit(right);
        if let (Some(l), Some(r)) = (left_value.number(), right_value.number()) {
            match operator.token_type {
                TokenType::Minus => return LiteralValue::Number(l - r),
                TokenType::Plus => return LiteralValue::Number(l + r),
                TokenType::Star => return LiteralValue::Number(l * r),
                TokenType::Slash => return LiteralValue::Number(l / r),
                TokenType::Greater => return LiteralValue::Boolean(l > r),
                TokenType::GreaterEqual => return LiteralValue::Boolean(l >= r),
                TokenType::Less => return LiteralValue::Boolean(l < r),
                TokenType::LessEqual => return LiteralValue::Boolean(l <= r),
                _ => {}
            }
        }

        // Both operands are strings
        if let (Some(l), Some(r)) = (left_value.string(), right_value.string()) {
            if operator.token_type == TokenType::Plus {
                return LiteralValue::String(format!("{}{}", l, r));
            }
        }

        if operator.token_type == TokenType::EqualEqual {
            return LiteralValue::Boolean(left_value == right_value);
        }
        if operator.token_type == TokenType::BangEqual {
            return LiteralValue::Boolean(left_value != right_value);
        }

        // Unreachable code
        LiteralValue::Null
    }
}
