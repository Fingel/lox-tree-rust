use crate::environment::Environment;
use crate::error_reporter::ErrorReporter;
use crate::expressions::Expr;
use crate::statements::Stmt;
use crate::tokens::{Object, Token, TokenType};

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
    pub token: Token,
}

pub struct Interpreter {
    pub error_reporter: ErrorReporter,
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            error_reporter: ErrorReporter::new(),
            environment: Environment::new(),
        }
    }
    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for statement in statements {
            if let Err(err) = self.execute(&statement) {
                self.error_reporter.runtime_error(err);
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Print(expr) => {
                let value = self.evaluate(expr)?;
                println!("{}", value);
                Ok(())
            }
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
                Ok(())
            }
            Stmt::Var(name, initializer) => {
                let value = if let Some(initializer) = initializer.as_ref() {
                    self.evaluate(initializer)?
                } else {
                    Object::Nil
                };
                self.environment.define(name.lexeme.clone(), value);
                Ok(())
            }
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Object, RuntimeError> {
        match expr {
            Expr::Literal(literal) => Ok(literal.clone()),
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Unary(op, right) => Ok(self.evaluate_unary(op, right)?),
            Expr::Binary(left, op, right) => Ok(self.evaluate_binary(left, op, right)?),
            Expr::Variable(name) => self.environment.get(name),
            Expr::Assignment(name, value) => self.evaluate_assignment(name, value),
        }
    }

    fn evaluate_assignment(&mut self, name: &Token, value: &Expr) -> Result<Object, RuntimeError> {
        let value = self.evaluate(value)?;
        self.environment.assign(name, value.clone())?;
        Ok(value)
    }

    fn evaluate_binary(
        &mut self,
        left: &Expr,
        op: &Token,
        right: &Expr,
    ) -> Result<Object, RuntimeError> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;
        match op.token_type {
            TokenType::Minus => {
                let (left_num, right_num) = self.check_number_operands(op, &left, &right)?;
                Ok(Object::Number(left_num - right_num))
            }
            TokenType::Slash => {
                let (left_num, right_num) = self.check_number_operands(op, &left, &right)?;
                Ok(Object::Number(left_num / right_num))
            }
            TokenType::Star => {
                let (left_num, right_num) = self.check_number_operands(op, &left, &right)?;
                Ok(Object::Number(left_num * right_num))
            }
            TokenType::Plus => match (left, right) {
                (Object::Number(left), Object::Number(right)) => Ok(Object::Number(left + right)),
                (Object::String(left), Object::String(right)) => {
                    Ok(Object::String(format!("{}{}", left, right)))
                }
                _ => Err(RuntimeError {
                    message: "Operands must be two numbers or two strings".to_string(),
                    token: op.clone(),
                }),
            },
            TokenType::Greater => {
                let (left_num, right_num) = self.check_number_operands(op, &left, &right)?;
                Ok(Object::Boolean(left_num > right_num))
            }
            TokenType::GreaterEqual => {
                let (left_num, right_num) = self.check_number_operands(op, &left, &right)?;
                Ok(Object::Boolean(left_num >= right_num))
            }
            TokenType::Less => {
                let (left_num, right_num) = self.check_number_operands(op, &left, &right)?;
                Ok(Object::Boolean(left_num < right_num))
            }
            TokenType::LessEqual => {
                let (left_num, right_num) = self.check_number_operands(op, &left, &right)?;
                Ok(Object::Boolean(left_num <= right_num))
            }
            TokenType::BangEqual => Ok(Object::Boolean(!self.is_equal(&left, &right))),
            TokenType::EqualEqual => Ok(Object::Boolean(self.is_equal(&left, &right))),
            _ => Err(RuntimeError {
                message: "Unhandled token type".to_string(),
                token: op.clone(),
            }),
        }
    }

    fn evaluate_unary(&mut self, operator: &Token, right: &Expr) -> Result<Object, RuntimeError> {
        let right = self.evaluate(right)?;
        match operator.token_type {
            TokenType::Minus => {
                let right_num = self.check_number_operand(operator, &right)?;
                Ok(Object::Number(-right_num))
            }
            TokenType::Bang => Ok(Object::Boolean(!self.is_truthy(&right))),
            _ => Err(RuntimeError {
                message: "Invalid operator".to_string(),
                token: operator.clone(),
            }),
        }
    }

    fn check_number_operand(
        &self,
        operator: &Token,
        operand: &Object,
    ) -> Result<f64, RuntimeError> {
        match operand {
            Object::Number(num) => Ok(*num),
            _ => Err(RuntimeError {
                message: "Operand must be a number".to_string(),
                token: operator.clone(),
            }),
        }
    }
    fn check_number_operands(
        &self,
        op: &Token,
        left: &Object,
        right: &Object,
    ) -> Result<(f64, f64), RuntimeError> {
        match (left, right) {
            (Object::Number(left), Object::Number(right)) => Ok((*left, *right)),
            _ => Err(RuntimeError {
                message: "Operands must be two numbers".to_string(),
                token: op.clone(),
            }),
        }
    }

    fn is_truthy(&self, value: &Object) -> bool {
        match value {
            Object::Nil => false,
            Object::Boolean(b) => *b,
            _ => true,
        }
    }

    fn is_equal(&self, a: &Object, b: &Object) -> bool {
        match (a, b) {
            (Object::Number(a), Object::Number(b)) => a == b,
            (Object::String(a), Object::String(b)) => a == b,
            (Object::Boolean(a), Object::Boolean(b)) => a == b,
            (Object::Nil, Object::Nil) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpret_addition() {
        // 1 + 2
        let mut interpreter = Interpreter::new();
        assert_eq!(
            interpreter
                .evaluate(&Expr::Binary(
                    Box::new(Expr::Literal(Object::Number(1.0))),
                    Token::new(TokenType::Plus, "+".to_string(), None, 1),
                    Box::new(Expr::Literal(Object::Number(2.0)))
                ))
                .unwrap(),
            Object::Number(3.0)
        );
    }

    #[test]
    fn test_equality() {
        let mut interpreter = Interpreter::new();
        assert_eq!(
            interpreter
                .evaluate(&Expr::Binary(
                    Box::new(Expr::Literal(Object::Number(1.0))),
                    Token::new(TokenType::EqualEqual, "==".to_string(), None, 1),
                    Box::new(Expr::Literal(Object::Number(1.0)))
                ))
                .unwrap(),
            Object::Boolean(true)
        );
    }
}
