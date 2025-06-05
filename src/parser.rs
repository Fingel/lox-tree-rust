use crate::error_reporter::ErrorReporter;
use crate::expressions::Expr;
use crate::statements::Stmt;
use crate::tokens::{Object, Token, TokenType};

#[derive(Debug)]
struct ParseError;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    pub error_reporter: ErrorReporter,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            error_reporter: ErrorReporter::new(),
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
            }
        }
        statements
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn declaration(&mut self) -> Option<Stmt> {
        let result = if self.match_token(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        match result {
            Ok(stmt) => Some(stmt),
            Err(_) => {
                self.synchronize();
                None
            }
        }
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(&[TokenType::Print]) {
            return self.print_statement();
        }

        self.expression_statement()
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self
            .consume(TokenType::Identifier, "Expect variable name.")?
            .clone();

        let initializer = if self.match_token(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        Ok(Stmt::Var(name, Box::new(initializer)))
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(Box::new(value)))
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Expression(Box::new(expr)))
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.equality()?;
        if self.match_token(&[TokenType::Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            match expr {
                Expr::Variable(name) => return Ok(Expr::Assignment(name, Box::new(value))),
                _ => _ = self.error(&equals, "Invalid assignment target"),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;
        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            // Pretty sure we want clone here as I think it makes sense
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;
        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;
        while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;
        while self.match_token(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary(operator, Box::new(right)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&[TokenType::False]) {
            return Ok(Expr::Literal(Object::Boolean(false)));
        }
        if self.match_token(&[TokenType::True]) {
            return Ok(Expr::Literal(Object::Boolean(true)));
        }
        if self.match_token(&[TokenType::Nil]) {
            return Ok(Expr::Literal(Object::Nil));
        }
        if self.match_token(&[TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal(self.previous().literal.clone().unwrap()));
        }
        if self.match_token(&[TokenType::Identifier]) {
            let token = self.previous().clone();
            return Ok(Expr::Variable(token));
        }
        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(Box::new(expr)));
        }

        Err(self.error(&self.peek().clone(), "Expect expression."))
    }

    fn error(&mut self, token: &Token, message: &str) -> ParseError {
        self.error_reporter.error_at_token(token, message);
        ParseError
    }

    fn consume(&mut self, type_: TokenType, message: &str) -> Result<&Token, ParseError> {
        if self.check(type_) {
            Ok(self.advance())
        } else {
            let token = &self.peek().clone();
            Err(self.error(token, message))
        }
    }

    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for type_ in types {
            if self.check(*type_) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, type_: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == type_
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::For
                | TokenType::Fun
                | TokenType::If
                | TokenType::Print
                | TokenType::Return
                | TokenType::Var
                | TokenType::While => return,
                _ => {}
            }

            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        // 1 + (2 * 3);
        let tokens = vec![
            Token::new(
                TokenType::Number,
                "1.0".to_string(),
                Some(Object::Number(1.0)),
                1,
            ),
            Token::new(TokenType::Plus, "+".to_string(), None, 1),
            Token::new(TokenType::LeftParen, "(".to_string(), None, 1),
            Token::new(
                TokenType::Number,
                "2.0".to_string(),
                Some(Object::Number(2.0)),
                1,
            ),
            Token::new(TokenType::Star, "*".to_string(), None, 1),
            Token::new(
                TokenType::Number,
                "3.0".to_string(),
                Some(Object::Number(3.0)),
                1,
            ),
            Token::new(TokenType::RightParen, ")".to_string(), None, 1),
            Token::new(TokenType::Semicolon, ";".to_string(), None, 1),
            Token::new(TokenType::Eof, "".to_string(), None, 1),
        ];

        let mut parser = Parser::new(tokens);
        let statements = parser.parse();
        let expected = Stmt::Expression(Box::new(Expr::Binary(
            Box::new(Expr::Literal(Object::Number(1.0))),
            Token::new(TokenType::Plus, "+".to_string(), None, 1),
            Box::new(Expr::Grouping(Box::new(Expr::Binary(
                Box::new(Expr::Literal(Object::Number(2.0))),
                Token::new(TokenType::Star, "*".to_string(), None, 1),
                Box::new(Expr::Literal(Object::Number(3.0))),
            )))),
        )));
        assert_eq!(statements[0], expected);
    }
}
