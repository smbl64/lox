use crate::prelude::*;

#[allow(unused)]
pub struct AstPrinter;

impl AstPrinter {
    #[allow(unused)]
    pub fn to_string(expr: &Expr) -> String {
        match expr {
            Expr::Binary { left, operator, right } => {
                format!(
                    "({} {} {})",
                    operator.lexeme,
                    Self::to_string(left),
                    Self::to_string(right)
                )
            }
            Expr::Grouping { expr } => format!("(group {})", Self::to_string(expr)),
            Expr::Literal { value } => format!("{value}"),
            Expr::Unary { operator, right } => {
                format!("({} {})", operator.lexeme, Self::to_string(right))
            }
            Expr::Variable { name } => format!("{name}"),
            Expr::Assignment { name, value } => format!("{name} = {}", Self::to_string(value)),
            Expr::Logical { left, operator, right } => {
                format!(
                    "({} {} {})",
                    operator.lexeme,
                    Self::to_string(left),
                    Self::to_string(right)
                )
            }
            Expr::Call { callee, paren: _, arguments } => {
                format!("{:?}({arguments:?})", Self::to_string(callee))
            }
            Expr::This { keyword } => format!("{keyword}"),
            Expr::Get { object, name } => format!("{:?}.{name}", Self::to_string(object)),
            Expr::Set { object, name, value } => {
                format!("{:?}.{name} = {:?}", Self::to_string(object), Self::to_string(value))
            }
            Expr::Super { keyword, method } => format!("{keyword}.{method}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenType;

    #[test]
    fn print_an_ast() {
        // This is '-123 * (45.67)'
        let expr = Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: Token::new(TokenType::Minus, "-", None, 1),
                right: Box::new(Expr::int_literal(123.0)),
            }),
            operator: Token::new(TokenType::Star, "*", None, 1),
            right: Box::new(Expr::Grouping { expr: Box::new(Expr::int_literal(45.67)) }),
        };

        let res = AstPrinter::to_string(&expr);
        assert_eq!(res, "(* (- 123) (group 45.67))".to_owned());
    }
}
