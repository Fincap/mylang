use crate::{
    error::lexer_error,
    token::{Token, TokenType},
};
use phf::*;

static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "and" => TokenType::And,
    "class" => TokenType::Class,
    "else" => TokenType::Else,
    "false" => TokenType::False,
    "fn" => TokenType::Fn,
    "for" => TokenType::For,
    "if" => TokenType::If,
    "let" => TokenType::Let,
    "null" => TokenType::Null,
    "or" => TokenType::Or,
    "print" => TokenType::Print,
    "return" => TokenType::Return,
    "super" => TokenType::Super,
    "this" => TokenType::This,
    "true" => TokenType::True,
    "while" => TokenType::While,
};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}
impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            // Beginning of next lexeme
            self.start = self.current;
            self.scan_token()
        }

        self.tokens.push(Token::new(
            TokenType::EOF,
            String::new(),
            self.tokens.last().map_or(1, |last| last.line),
        ));
        self.tokens.to_owned()
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            ';' => self.add_token(TokenType::Semicolon),
            '+' => {
                if self.match_next('=') {
                    self.add_token(TokenType::PlusEqual)
                } else if self.match_next('+') {
                    self.add_token(TokenType::PlusPlus)
                } else {
                    self.add_token(TokenType::Plus)
                }
            }
            '-' => {
                if self.match_next('=') {
                    self.add_token(TokenType::MinusEqual)
                } else if self.match_next('-') {
                    self.add_token(TokenType::MinusMinus)
                } else {
                    self.add_token(TokenType::Minus)
                }
            }
            '*' => {
                if self.match_next('=') {
                    self.add_token(TokenType::StarEqual)
                } else {
                    self.add_token(TokenType::Star)
                }
            }
            '!' => {
                if self.match_next('=') {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                }
            }
            '=' => {
                if self.match_next('=') {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            }
            '<' => {
                if self.match_next('=') {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::Less)
                }
            }
            '>' => {
                if self.match_next('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::Greater)
                }
            }
            '/' => {
                if self.match_next('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.match_next('*') {
                    while !self.is_at_end() {
                        if self.peek() == '\n' {
                            self.line += 1;
                        }
                        if self.advance() == '*' && self.peek() == '/' {
                            self.advance();
                            break;
                        }
                    }
                } else if self.match_next('=') {
                    self.add_token(TokenType::SlashEqual)
                } else {
                    self.add_token(TokenType::Slash)
                }
            }
            '"' => self.scan_string(),
            '0'..='9' => self.scan_number(),
            'a'..='z' | 'A'..='Z' | '_' => self.scan_identifier(),
            _ => lexer_error(self.line, format!("Unexpected character {}", c)),
        }
    }

    fn scan_string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            lexer_error(self.line, String::from("Unterminated string"));
            return;
        }
        self.advance(); // consume the closing "
        let value = String::from(&self.source[self.start + 1..self.current - 1]);
        self.add_token(TokenType::String(value));
    }

    fn scan_number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // Look for fractional part
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // Consume the "."
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        self.add_token(TokenType::Number(
            self.source[self.start..self.current]
                .parse::<f64>()
                .unwrap(),
        ));
    }

    fn scan_identifier(&mut self) {
        while Scanner::is_alphanumeric(self.peek()) {
            self.advance();
        }
        let text = &self.source[self.start..self.current];
        let t_type = match KEYWORDS.get(text) {
            Some(keyword) => keyword.to_owned(),
            None => TokenType::Identifier,
        };
        self.add_token(t_type);
    }

    fn advance(&mut self) -> char {
        let res = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        res
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap()
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).unwrap()
        }
    }

    fn add_token(&mut self, p_type: TokenType) {
        let text = &self.source[self.start..self.current];
        self.tokens
            .push(Token::new(p_type, String::from(text), self.line));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn is_alphanumeric(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }
}
