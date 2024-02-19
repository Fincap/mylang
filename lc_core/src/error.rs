use crate::token::{TokenError, TokenKind};
use anyhow::anyhow;

pub fn lexer_error(line: usize, message: String) {
    report(line, "".to_string(), message);
}

pub fn parser_error(err: TokenError) {
    if let TokenKind::EOF = err.token.kind {
        report(err.token.line, " at end".to_string(), err.message);
    } else {
        report(
            err.token.line,
            format!(" at '{}'", err.token.lexeme),
            err.message,
        );
    }
}

pub fn runtime_error(err: TokenError) -> anyhow::Result<()> {
    Err(anyhow!(
        "[line {}] RuntimeError: {}",
        err.token.line,
        err.message
    ))
}

fn report(line: usize, loc: String, message: String) {
    eprintln!("[line {}] Error{}: {}", line, loc, message);
}
