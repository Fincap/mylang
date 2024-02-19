use std::hash::Hash;

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

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
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
    pub fn new(t_type: TokenKind, lexeme: String, line: usize) -> Self {
        Self {
            kind: t_type,
            lexeme,
            line,
        }
    }

    pub fn as_str(&self) -> String {
        format!("{} {:?}", self.lexeme, self.kind)
    }
}
