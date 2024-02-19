use std::{error, fmt};

use anyhow::Error;

use crate::Token;

pub type SpanMessage = (usize, String);
pub type TranslationResult<T> = (T, TranslationErrors);

#[derive(Debug, Clone)]
pub struct TranslationErrors {
    issues: Vec<SpanMessage>,
}
impl fmt::Display for TranslationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (line, message) in &self.issues {
            writeln!(f, "[line {}] TranslationError: {}", line, message)?;
        }
        Ok(())
    }
}
impl error::Error for TranslationErrors {}
impl From<Vec<TokenError>> for TranslationErrors {
    fn from(issues: Vec<TokenError>) -> Self {
        Self {
            issues: issues
                .iter()
                .map(|t| (t.token.span.line, t.message.to_owned()))
                .collect(),
        }
    }
}
impl From<Vec<SpanMessage>> for TranslationErrors {
    fn from(issues: Vec<SpanMessage>) -> Self {
        Self { issues }
    }
}
impl<'a> TranslationErrors {
    pub fn new() -> Self {
        Self { issues: Vec::new() }
    }

    pub fn merge(&mut self, other: &mut TranslationErrors) {
        self.issues.append(&mut other.issues);
    }

    pub fn has_errors(&self) -> bool {
        !self.issues.is_empty()
    }

    pub fn check(&'a self) -> Result<(), Error> {
        if self.has_errors() {
            Err(self.to_owned().into())
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    line: usize,
    message: String,
}
impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[line {}] RuntimeError: {}", self.line, self.message)
    }
}
impl error::Error for RuntimeError {}
impl From<TokenError> for RuntimeError {
    fn from(value: TokenError) -> Self {
        Self {
            line: value.token.span.line,
            message: value.message,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TokenError {
    pub token: Token,
    pub message: String,
}
impl error::Error for TokenError {}
impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl From<(&Token, &str)> for TokenError {
    fn from(value: (&Token, &str)) -> Self {
        Self {
            token: value.0.to_owned(),
            message: value.1.to_string(),
        }
    }
}
impl From<(&Token, String)> for TokenError {
    fn from(value: (&Token, String)) -> Self {
        Self {
            token: value.0.to_owned(),
            message: value.1,
        }
    }
}
