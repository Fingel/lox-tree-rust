use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::path::Path;
use std::process::exit;

mod callable;
mod environment;
mod error_reporter;
mod expressions;
mod interpreter;
mod parser;
mod scanner;
mod statements;
mod tokens;

use error_reporter::ErrorReporter;
use interpreter::Interpreter;
use parser::Parser;
use scanner::Scanner;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        println!("Usage: rlox <script>");
    } else if args.len() == 2 {
        run_file(Path::new(&args[1]))?;
    } else {
        run_prompt()?;
    }

    Ok(())
}

fn run_file(path: &Path) -> Result<(), io::Error> {
    let contents = std::fs::read_to_string(path)?;
    run(contents);
    Ok(())
}

fn run_prompt() -> Result<(), io::Error> {
    loop {
        let mut line = String::new();
        print!("> ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut line)?;
        if line.trim().is_empty() {
            break;
        }
        run(line);
    }
    Ok(())
}

fn run(source: String) {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    check_errors(&scanner.error_reporter);
    let mut parser = Parser::new(tokens);
    let statements = parser.parse();
    check_errors(&parser.error_reporter);
    let mut interpreter = Interpreter::new();
    interpreter.interpret(statements);
    check_errors(&interpreter.error_reporter);
}

fn check_errors(error_reporter: &ErrorReporter) {
    if error_reporter.had_error {
        exit(65);
    }
    if error_reporter.had_runtime_error {
        exit(70);
    }
}
