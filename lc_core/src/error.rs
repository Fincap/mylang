use std::{error, fmt::Display};

use anyhow::Error;

use crate::token::TokenError;

pub type SpanMessage = (usize, String);
pub type TranslationResult<T> = (T, TranslationErrors);

#[derive(Debug, Clone)]
pub struct TranslationErrors {
    issues: Vec<SpanMessage>,
}
impl Display for TranslationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
                .map(|t| (t.token.line, t.message.to_owned()))
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
impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[line {}] RuntimeError: {}", self.line, self.message)
    }
}
impl error::Error for RuntimeError {}
impl From<TokenError> for RuntimeError {
    fn from(value: TokenError) -> Self {
        Self {
            line: value.token.line,
            message: value.message,
        }
    }
}
