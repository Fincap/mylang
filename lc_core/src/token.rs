use std::{
    error,
    fmt::{self, Display},
    hash::Hash,
};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
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

#[derive(Clone, Debug)]
pub struct Token {
    pub t_type: TokenType,
    pub lexeme: String,
    pub line: usize,
}
impl Hash for Token {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.lexeme.hash(state);
        self.line.hash(state);
    }
}
impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.lexeme == other.lexeme && self.line == other.line
    }
}
impl Eq for Token {}
impl Token {
    pub fn new(t_type: TokenType, lexeme: String, line: usize) -> Self {
        Self {
            t_type,
            lexeme,
            line,
        }
    }

    pub fn as_str(&self) -> String {
        format!("{} {:?}", self.lexeme, self.t_type)
    }
}

#[derive(Clone, Debug)]
pub struct TokenError {
    pub token: Token,
    pub message: String,
}
impl error::Error for TokenError {}
impl Display for TokenError {
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
