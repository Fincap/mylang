use crate::{
    token::{Token, TokenKind},
    Span, SpanMessage, TranslationResult,
};
use phf::*;

static KEYWORDS: phf::Map<&'static str, TokenKind> = phf_map! {
    "and" => TokenKind::And,
    "class" => TokenKind::Class,
    "else" => TokenKind::Else,
    "false" => TokenKind::False,
    "fn" => TokenKind::Fn,
    "for" => TokenKind::For,
    "if" => TokenKind::If,
    "let" => TokenKind::Let,
    "null" => TokenKind::Null,
    "or" => TokenKind::Or,
    "print" => TokenKind::Print,
    "return" => TokenKind::Return,
    "super" => TokenKind::Super,
    "this" => TokenKind::This,
    "true" => TokenKind::True,
    "while" => TokenKind::While,
};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    errors: Vec<SpanMessage>,
}
impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            errors: Vec::new(),
        }
    }

    pub fn scan_tokens(&mut self) -> TranslationResult<Vec<Token>> {
        while !self.is_at_end() {
            // Beginning of next lexeme
            self.start = self.current;
            self.scan_token()
        }

        self.tokens.push(Token::new(
            TokenKind::EOF,
            String::new(),
            self.tokens
                .last()
                .map_or(Span::new(1), |last| Span::new(last.span.line)),
        ));
        (self.tokens.to_owned(), self.errors.clone().into())
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '(' => self.add_token(TokenKind::LeftParen),
            ')' => self.add_token(TokenKind::RightParen),
            '{' => self.add_token(TokenKind::LeftBrace),
            '}' => self.add_token(TokenKind::RightBrace),
            ',' => self.add_token(TokenKind::Comma),
            '.' => self.add_token(TokenKind::Dot),
            ';' => self.add_token(TokenKind::Semicolon),
            '+' => {
                if self.match_next('=') {
                    self.add_token(TokenKind::PlusEqual)
                } else if self.match_next('+') {
                    self.add_token(TokenKind::PlusPlus)
                } else {
                    self.add_token(TokenKind::Plus)
                }
            }
            '-' => {
                if self.match_next('=') {
                    self.add_token(TokenKind::MinusEqual)
                } else if self.match_next('-') {
                    self.add_token(TokenKind::MinusMinus)
                } else {
                    self.add_token(TokenKind::Minus)
                }
            }
            '*' => {
                if self.match_next('=') {
                    self.add_token(TokenKind::StarEqual)
                } else {
                    self.add_token(TokenKind::Star)
                }
            }
            '!' => {
                if self.match_next('=') {
                    self.add_token(TokenKind::BangEqual)
                } else {
                    self.add_token(TokenKind::Bang)
                }
            }
            '=' => {
                if self.match_next('=') {
                    self.add_token(TokenKind::EqualEqual)
                } else {
                    self.add_token(TokenKind::Equal)
                }
            }
            '<' => {
                if self.match_next('=') {
                    self.add_token(TokenKind::LessEqual)
                } else {
                    self.add_token(TokenKind::Less)
                }
            }
            '>' => {
                if self.match_next('=') {
                    self.add_token(TokenKind::GreaterEqual)
                } else {
                    self.add_token(TokenKind::Greater)
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
                    self.add_token(TokenKind::SlashEqual)
                } else {
                    self.add_token(TokenKind::Slash)
                }
            }
            '"' => self.scan_string(),
            '0'..='9' => self.scan_number(),
            'a'..='z' | 'A'..='Z' | '_' => self.scan_identifier(),
            _ => self.report_error(self.line, format!("Unexpected character {}", c)),
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
            self.report_error(self.line, String::from("Unterminated string"));
            return;
        }
        self.advance(); // consume the closing "
        let value = String::from(&self.source[self.start + 1..self.current - 1]);
        self.add_token(TokenKind::String(value));
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

        self.add_token(TokenKind::Number(
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
            None => TokenKind::Identifier,
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

    fn add_token(&mut self, p_type: TokenKind) {
        let text = &self.source[self.start..self.current];
        self.tokens
            .push(Token::new(p_type, String::from(text), Span::new(self.line)));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn is_alphanumeric(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    fn report_error(&mut self, line: usize, message: String) {
        self.errors.push((line, message));
    }
}
