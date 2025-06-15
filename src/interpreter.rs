use crate::environment::EnvironmentStack;
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
    environment: EnvironmentStack,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            error_reporter: ErrorReporter::new(),
            environment: EnvironmentStack::new(),
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
            // these map the "visit<type>Stmt" functions from the book
            Stmt::Print(expr) => self.execute_print_statement(expr),
            Stmt::Expression(expr) => self.execute_expression_statement(expr),
            Stmt::Var(name, initializer) => self.execute_var_statement(name, initializer),
            Stmt::Block(statements) => self.execute_block_statement(statements),
            Stmt::If(condition, then_branch, else_branch) => {
                self.execute_if_statement(condition, then_branch, else_branch)
            }
            Stmt::While(condition, body) => self.execute_while_statement(condition, body),
        }
    }

    // visitExpressionStmt
    fn execute_expression_statement(&mut self, expr: &Expr) -> Result<(), RuntimeError> {
        self.evaluate(expr)?;
        Ok(())
    }

    // visitPrintStmt
    fn execute_print_statement(&mut self, expr: &Expr) -> Result<(), RuntimeError> {
        let value = self.evaluate(expr)?;
        println!("{}", value);
        Ok(())
    }

    //visitBlockStmt
    fn execute_block_statement(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        self.execute_block(statements)
    }

    //visitIfStmt
    fn execute_if_statement(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Box<Stmt>>,
    ) -> Result<(), RuntimeError> {
        let condition_value = self.evaluate(condition)?;
        if self.is_truthy(&condition_value) {
            self.execute(then_branch)?;
        } else if let Some(else_branch) = else_branch.as_ref() {
            self.execute(else_branch)?;
        }
        Ok(())
    }

    fn execute_block(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        // Create a new environment for the block
        self.environment.push_environment();

        for statement in statements {
            if let Err(e) = self.execute(statement) {
                self.environment.pop_environment();
                return Err(e);
            }
        }

        self.environment.pop_environment();

        Ok(())
    }

    // visitVarStmt
    fn execute_var_statement(
        &mut self,
        name: &Token,
        initializer: &Option<Box<Expr>>,
    ) -> Result<(), RuntimeError> {
        let value = if let Some(initializer) = initializer.as_ref() {
            self.evaluate(initializer)?
        } else {
            Object::Nil
        };
        self.environment.define(name, value);
        Ok(())
    }

    //visitWhileStmt
    fn execute_while_statement(
        &mut self,
        condition: &Expr,
        body: &Stmt,
    ) -> Result<(), RuntimeError> {
        loop {
            let condition_val = self.evaluate(condition)?;
            if !self.is_truthy(&condition_val) {
                break;
            }
            self.execute(body)?;
        }
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Object, RuntimeError> {
        match expr {
            // These map the "visit<type>Expr" methods from the book
            Expr::Literal(literal) => self.evaluate_literal_expr(literal),
            Expr::Grouping(expr) => self.evaluate_grouping_expr(expr),
            Expr::Unary(op, right) => self.evaluate_unary_expr(op, right),
            Expr::Binary(left, op, right) => self.evaluate_binary_expr(left, op, right),
            Expr::Variable(name) => self.evaluate_variable_expr(name),
            Expr::Assignment(name, value) => self.evaluate_assignment_expr(name, value),
            Expr::Logical(left, op, right) => self.evaluate_logical_expr(left, op, right),
        }
    }

    // visitAssignmentExpr
    fn evaluate_assignment_expr(
        &mut self,
        name: &Token,
        value: &Expr,
    ) -> Result<Object, RuntimeError> {
        let value = self.evaluate(value)?;
        self.environment.assign(name, value.clone())?;
        Ok(value)
    }

    //visitBinaryExpr
    fn evaluate_binary_expr(
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

    // visitGroupingExpr
    fn evaluate_grouping_expr(&mut self, expr: &Expr) -> Result<Object, RuntimeError> {
        self.evaluate(expr)
    }

    // visitLiteralExpr
    fn evaluate_literal_expr(&mut self, literal: &Object) -> Result<Object, RuntimeError> {
        Ok(literal.clone())
    }

    // visitLogicalExpr
    fn evaluate_logical_expr(
        &mut self,
        left: &Expr,
        op: &Token,
        right: &Expr,
    ) -> Result<Object, RuntimeError> {
        let left_expr = self.evaluate(left)?;
        if op.token_type == TokenType::Or {
            if self.is_truthy(&left_expr) {
                return Ok(left_expr);
            }
        } else if !self.is_truthy(&left_expr) {
            return Ok(left_expr);
        }
        self.evaluate(right)
    }

    // visitUnaryExpr
    fn evaluate_unary_expr(
        &mut self,
        operator: &Token,
        right: &Expr,
    ) -> Result<Object, RuntimeError> {
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

    // visitVariableExpr
    fn evaluate_variable_expr(&mut self, name: &Token) -> Result<Object, RuntimeError> {
        self.environment.get(name)
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

    #[test]
    fn test_interpret_variable_declaration_and_usage() {
        let mut interpreter = Interpreter::new();
        let var_name = Token::new(TokenType::Identifier, "test_var".to_string(), None, 1);

        let statements = vec![
            // var test_var = 123;
            Stmt::Var(
                var_name.clone(),
                Some(Box::new(Expr::Literal(Object::Number(123.0)))),
            ),
            // print test_var;
            Stmt::Print(Box::new(Expr::Variable(var_name.clone()))),
        ];

        interpreter.interpret(statements);

        // Should not have any errors
        assert!(!interpreter.error_reporter.had_runtime_error);
        // Variable should exist in environment
        assert_eq!(
            interpreter.environment.get(&var_name).unwrap(),
            Object::Number(123.0)
        );
    }

    #[test]
    fn test_interpret_variable_reassignment() {
        let mut interpreter = Interpreter::new();
        let var_name = Token::new(TokenType::Identifier, "test_var".to_string(), None, 1);

        let statements = vec![
            // var test_var = 123;
            Stmt::Var(
                var_name.clone(),
                Some(Box::new(Expr::Literal(Object::Number(123.0)))),
            ),
            // var test_var = 42;
            Stmt::Var(
                var_name.clone(),
                Some(Box::new(Expr::Literal(Object::Number(42.0)))),
            ),
        ];

        interpreter.interpret(statements);

        // Should not have any errors
        assert!(!interpreter.error_reporter.had_runtime_error);
        // Variable should exist in environment
        assert_eq!(
            interpreter.environment.get(&var_name).unwrap(),
            Object::Number(42.0)
        );
    }

    #[test]
    fn test_block_statement_scoping_and_shadowing() {
        let mut interpreter = Interpreter::new();

        let var_a = Token::new(TokenType::Identifier, "a".to_string(), None, 1);
        let var_b = Token::new(TokenType::Identifier, "b".to_string(), None, 1);

        let statements = vec![
            // var a = "global a";
            Stmt::Var(
                var_a.clone(),
                Some(Box::new(Expr::Literal(Object::String(
                    "global a".to_string(),
                )))),
            ),
            // var b = "global b";
            Stmt::Var(
                var_b.clone(),
                Some(Box::new(Expr::Literal(Object::String(
                    "global b".to_string(),
                )))),
            ),
            // {
            //   var a = "outer a";
            //   var b = "outer b";
            // }
            Stmt::Block(vec![
                Stmt::Var(
                    var_a.clone(),
                    Some(Box::new(Expr::Literal(Object::String(
                        "outer a".to_string(),
                    )))),
                ),
                Stmt::Var(
                    var_b.clone(),
                    Some(Box::new(Expr::Literal(Object::String(
                        "outer b".to_string(),
                    )))),
                ),
            ]),
        ];

        interpreter.interpret(statements);

        // Should not have any errors
        assert!(!interpreter.error_reporter.had_runtime_error);

        // After all blocks have closed, variables should have their global values
        assert_eq!(
            interpreter.environment.get(&var_a).unwrap(),
            Object::String("global a".to_string())
        );
        assert_eq!(
            interpreter.environment.get(&var_b).unwrap(),
            Object::String("global b".to_string())
        );
    }

    #[test]
    fn test_block_scope_isolation() {
        let mut interpreter = Interpreter::new();

        let var_block_only = Token::new(TokenType::Identifier, "block_only".to_string(), None, 1);

        let statements = vec![
            // {
            //   var block_only = "inside block";
            // }
            Stmt::Block(vec![Stmt::Var(
                var_block_only.clone(),
                Some(Box::new(Expr::Literal(Object::String(
                    "inside block".to_string(),
                )))),
            )]),
            // Try to access block_only variable outside the block - this should cause an error
            Stmt::Print(Box::new(Expr::Variable(var_block_only.clone()))),
        ];

        interpreter.interpret(statements);

        // Should have a runtime error because block_only is not accessible outside the block
        assert!(interpreter.error_reporter.had_runtime_error);
    }

    #[test]
    fn test_logical_and_short_circuit_false() {
        // Test that "and" short-circuits when left operand is false
        let mut interpreter = Interpreter::new();

        // false and true should return false without evaluating true
        let result = interpreter
            .evaluate(&Expr::Logical(
                Box::new(Expr::Literal(Object::Boolean(false))),
                Token::new(TokenType::And, "and".to_string(), None, 1),
                Box::new(Expr::Literal(Object::Boolean(true))),
            ))
            .unwrap();

        assert_eq!(result, Object::Boolean(false));
    }

    #[test]
    fn test_logical_and_evaluate_both() {
        // Test that "and" evaluates right operand when left is truthy
        let mut interpreter = Interpreter::new();

        // true and false should return false
        let result = interpreter
            .evaluate(&Expr::Logical(
                Box::new(Expr::Literal(Object::Boolean(true))),
                Token::new(TokenType::And, "and".to_string(), None, 1),
                Box::new(Expr::Literal(Object::Boolean(false))),
            ))
            .unwrap();

        assert_eq!(result, Object::Boolean(false));
    }

    #[test]
    fn test_logical_or_short_circuit_true() {
        // Test that "or" short-circuits when left operand is truthy
        let mut interpreter = Interpreter::new();

        // true or false should return true without evaluating false
        let result = interpreter
            .evaluate(&Expr::Logical(
                Box::new(Expr::Literal(Object::Boolean(true))),
                Token::new(TokenType::Or, "or".to_string(), None, 1),
                Box::new(Expr::Literal(Object::Boolean(false))),
            ))
            .unwrap();

        assert_eq!(result, Object::Boolean(true));
    }

    #[test]
    fn test_logical_or_evaluate_both() {
        // Test that "or" evaluates right operand when left is falsy
        let mut interpreter = Interpreter::new();

        // false or true should return true
        let result = interpreter
            .evaluate(&Expr::Logical(
                Box::new(Expr::Literal(Object::Boolean(false))),
                Token::new(TokenType::Or, "or".to_string(), None, 1),
                Box::new(Expr::Literal(Object::Boolean(true))),
            ))
            .unwrap();

        assert_eq!(result, Object::Boolean(true));
    }

    #[test]
    fn test_while_loop_with_blocks() {
        // Test that while loops work correctly with variable assignments in blocks
        let mut interpreter = Interpreter::new();

        let var_a = Token::new(TokenType::Identifier, "a".to_string(), None, 1);

        let statements = vec![
            // var a = 0;
            Stmt::Var(
                var_a.clone(),
                Some(Box::new(Expr::Literal(Object::Number(0.0)))),
            ),
            // while (a < 3) {
            //     a = a + 1;
            // }
            Stmt::While(
                Box::new(Expr::Binary(
                    Box::new(Expr::Variable(var_a.clone())),
                    Token::new(TokenType::Less, "<".to_string(), None, 1),
                    Box::new(Expr::Literal(Object::Number(3.0))),
                )),
                Box::new(Stmt::Block(vec![Stmt::Expression(Box::new(
                    Expr::Assignment(
                        var_a.clone(),
                        Box::new(Expr::Binary(
                            Box::new(Expr::Variable(var_a.clone())),
                            Token::new(TokenType::Plus, "+".to_string(), None, 1),
                            Box::new(Expr::Literal(Object::Number(1.0))),
                        )),
                    ),
                ))])),
            ),
        ];

        interpreter.interpret(statements);

        // Should not have any errors
        assert!(!interpreter.error_reporter.had_runtime_error);
        // Variable should have been incremented to 3
        assert_eq!(
            interpreter.environment.get(&var_a).unwrap(),
            Object::Number(3.0)
        );
    }
}
