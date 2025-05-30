#[derive(Default)]
pub struct ErrorReporter {
    pub had_error: bool,
}

#[allow(dead_code)]
impl ErrorReporter {
    pub fn error(&mut self, line: u32, message: &str) {
        self.report(line, "", message);
    }

    fn report(&mut self, line: u32, loc: &str, message: &str) {
        eprintln!("[line {}] Error {}: {}", line, loc, message);
        self.had_error = true;
    }
}
