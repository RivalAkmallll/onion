#![allow(dead_code)]

pub mod ast;
pub mod checker;
pub mod codegen;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod linker;
pub mod parser;

use error::OnionError;

/// Runs lexer, parser and checker. Returns every error found (empty = all good).
/// The language server uses this for red squiggles.
pub fn analyze(source: &str) -> Vec<OnionError> {
    let tokens = match lexer::lex(source) {
        Ok(tokens) => tokens,
        Err(e) => return vec![e],
    };
    let program = match parser::parse(tokens, source.len()) {
        Ok(program) => program,
        Err(e) => return vec![e],
    };
    match checker::check(&program) {
        Ok(()) => Vec::new(),
        Err(errors) => errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_program_has_no_diagnostics() {
        assert!(analyze("fn main() { println!(\"hi\") }").is_empty());
    }

    #[test]
    fn broken_program_reports_errors() {
        assert!(!analyze("fn main() { let x = 1 x = 2 }").is_empty());
    }
}
