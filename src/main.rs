use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::path::Path;
use std::process::exit;

mod error_reporter;
mod scanner;
mod tokens;

use error_reporter::ErrorReporter;

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
    run(&contents);
    Ok(())
}

fn run_prompt() -> Result<(), io::Error> {
    let mut line = String::new();
    loop {
        print!("> ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut line)?;
        run(&line);
        if line.trim().is_empty() {
            break;
        }
        line.clear();
    }
    Ok(())
}

fn run(source: &str) {
    print!("{}", source);
}

#[allow(dead_code)]
fn check_errors(error_reporter: &ErrorReporter) {
    if error_reporter.had_error {
        exit(65);
    }
}
