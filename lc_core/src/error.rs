use std::{error, fmt};

use anyhow::Error;

use crate::{Span, Token};

pub type SpannedMessage = (Span, String);
pub type TranslationResult<T> = (T, TranslationErrors);

#[derive(Default, Debug, Clone)]
pub struct TranslationErrors {
    issues: Vec<SpannedError>,
}
impl fmt::Display for TranslationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for issue in &self.issues {
            writeln!(
                f,
                "[line {}] TranslationError: {}",
                issue.span.line, issue.message
            )?;
        }
        Ok(())
    }
}
impl error::Error for TranslationErrors {}
impl From<Vec<SpannedError>> for TranslationErrors {
    fn from(issues: Vec<SpannedError>) -> Self {
        Self { issues }
    }
}
impl From<Vec<SpannedMessage>> for TranslationErrors {
    fn from(issues: Vec<SpannedMessage>) -> Self {
        Self {
            issues: issues.iter().map(|i| i.clone().into()).collect(),
        }
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

#[derive(Clone, Debug)]
pub struct RuntimeError {
    span: Option<Span>,
    message: String,
}
impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut line = String::new();
        if let Some(span) = self.span {
            line = format!("[line {}] ", span.line);
        }
        writeln!(f, "{}RuntimeError: {}", line, self.message)
    }
}
impl error::Error for RuntimeError {}
impl From<SpannedError> for RuntimeError {
    fn from(value: SpannedError) -> Self {
        Self {
            span: Some(value.span),
            message: value.message,
        }
    }
}
impl RuntimeError {
    pub fn new(message: String) -> Self {
        Self {
            span: None,
            message,
        }
    }

    pub fn with_span(message: String, span: Span) -> Self {
        Self {
            span: Some(span),
            message,
        }
    }

    pub fn has_span(&self) -> bool {
        self.span.is_some()
    }

    pub fn set_span(&mut self, span: Span) {
        self.span = Some(span)
    }
}

#[derive(Clone, Debug)]
pub struct SpannedError {
    pub span: Span,
    pub message: String,
}
impl error::Error for SpannedError {}
impl fmt::Display for SpannedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl From<(&Token, &str)> for SpannedError {
    fn from(value: (&Token, &str)) -> Self {
        Self {
            span: value.0.span.to_owned(),
            message: value.1.to_string(),
        }
    }
}
impl From<(&Token, String)> for SpannedError {
    fn from(value: (&Token, String)) -> Self {
        Self {
            span: value.0.span.to_owned(),
            message: value.1,
        }
    }
}
impl From<(Span, &str)> for SpannedError {
    fn from(value: (Span, &str)) -> Self {
        Self {
            span: value.0.to_owned(),
            message: value.1.to_string(),
        }
    }
}
impl From<(Span, String)> for SpannedError {
    fn from(value: (Span, String)) -> Self {
        Self {
            span: value.0.to_owned(),
            message: value.1,
        }
    }
}
