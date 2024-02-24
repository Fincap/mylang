use std::{cmp, hash::Hash};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Identifier,
    String(String),
    Number(f64),
    // Single character
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Semicolon,
    // One or two characters
    Minus,
    MinusEqual,
    MinusMinus,
    Plus,
    PlusEqual,
    PlusPlus,
    Slash,
    SlashEqual,
    Star,
    StarEqual,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Keywords
    And,
    Class,
    Else,
    False,
    Fn,
    For,
    If,
    Let,
    Null,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    While,
    EOF,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    pub line: usize,
    pub start: usize,
    pub end: usize,
}
impl Span {
    pub fn new(line: usize, start: usize, end: usize) -> Self {
        Self { line, start, end }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn to(&self, end: Span) -> Span {
        Span::new(
            cmp::min(self.line, end.line),
            cmp::min(self.start, end.start),
            cmp::max(self.end, end.end),
        )
    }

    pub fn between(&self, end: Span) -> Span {
        Span::new(
            cmp::min(self.line, end.line),
            cmp::max(self.end, end.end),
            cmp::min(self.start, end.start),
        )
    }

    pub fn until(&self, end: Span) -> Span {
        Span::new(
            cmp::min(self.line, end.line),
            cmp::min(self.start, end.start),
            cmp::max(self.start, end.start),
        )
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: Span,
}
impl Hash for Token {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.lexeme.hash(state);
        self.span.hash(state);
    }
}
impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.lexeme == other.lexeme && self.span == other.span
    }
}
impl Eq for Token {}
impl Token {
    pub fn new(t_type: TokenKind, lexeme: String, span: Span) -> Self {
        Self {
            kind: t_type,
            lexeme,
            span,
        }
    }

    pub fn as_str(&self) -> String {
        format!("{} {:?}", self.lexeme, self.kind)
    }
}
