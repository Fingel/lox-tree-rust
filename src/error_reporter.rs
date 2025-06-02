use crate::interpreter::RuntimeError;
use crate::tokens::{Token, TokenType};

pub struct ErrorReporter {
    pub had_error: bool,
    pub had_runtime_error: bool,
}

impl ErrorReporter {
    pub fn new() -> Self {
        ErrorReporter {
            had_error: false,
            had_runtime_error: false,
        }
    }

    pub fn error(&mut self, line: u32, message: &str) {
        self.report(line, "", message);
    }

    pub fn runtime_error(&mut self, error: RuntimeError) {
        eprintln!("{} \n[line {}]", error.message, error.token.line);
        self.had_runtime_error = true;
    }

    pub fn error_at_token(&mut self, token: &Token, message: &str) {
        if token.token_type == TokenType::Eof {
            self.report(token.line, " at end", message);
        } else {
            self.report(token.line, &format!(" at '{}'", token.lexeme), message);
        }
    }

    fn report(&mut self, line: u32, loc: &str, message: &str) {
        eprintln!("[line {}] Error {}: {}", line, loc, message);
        self.had_error = true;
    }
}
