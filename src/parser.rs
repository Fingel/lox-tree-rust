use crate::error_reporter::ErrorReporter;
use crate::expressions::Expr;
use crate::tokens::{Literal, Token, TokenType};

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

    pub fn parse(&mut self) -> Option<Expr> {
        self.expression().ok()
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
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
            return Ok(Expr::Literal(Literal::Boolean(false)));
        }
        if self.match_token(&[TokenType::True]) {
            return Ok(Expr::Literal(Literal::Boolean(true)));
        }
        if self.match_token(&[TokenType::Nil]) {
            return Ok(Expr::Literal(Literal::Nil));
        }
        if self.match_token(&[TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal(self.previous().literal.clone().unwrap()));
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

    #[allow(dead_code)]
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
                Some(Literal::Number(1.0)),
                1,
            ),
            Token::new(TokenType::Plus, "+".to_string(), None, 1),
            Token::new(TokenType::LeftParen, "(".to_string(), None, 1),
            Token::new(
                TokenType::Number,
                "2.0".to_string(),
                Some(Literal::Number(2.0)),
                1,
            ),
            Token::new(TokenType::Star, "*".to_string(), None, 1),
            Token::new(
                TokenType::Number,
                "3.0".to_string(),
                Some(Literal::Number(3.0)),
                1,
            ),
            Token::new(TokenType::RightParen, ")".to_string(), None, 1),
            Token::new(TokenType::Semicolon, ";".to_string(), None, 1),
        ];

        let mut parser = Parser::new(tokens);
        let expr = parser.parse().unwrap();
        let expected = Expr::Binary(
            Box::new(Expr::Literal(Literal::Number(1.0))),
            Token::new(TokenType::Plus, "+".to_string(), None, 1),
            Box::new(Expr::Grouping(Box::new(Expr::Binary(
                Box::new(Expr::Literal(Literal::Number(2.0))),
                Token::new(TokenType::Star, "*".to_string(), None, 1),
                Box::new(Expr::Literal(Literal::Number(3.0))),
            )))),
        );
        assert_eq!(format!("{}", expr), format!("{}", expected));
    }
}
