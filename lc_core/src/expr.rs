use std::hash::{Hash, Hasher};
use std::mem;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::token::Token;
use crate::{Span, TokenKind};

pub const LIMIT_FN_ARGS: usize = 255;
static EXPR_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum ExprKind {
    /// (`identifier`, `initializer`)
    Assign(Ident, Box<Expr>),
    /// (`left`, `op`, `right`)
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    /// (`callee`, `paren`, `args`)
    Call(Box<Expr>, Span, Vec<Expr>),
    /// (`expression`)
    Grouping(Box<Expr>),
    /// (`literal`)
    Literal(Literal),
    /// (`left`, `op`, `right`)
    Logical(Box<Expr>, Token, Box<Expr>),
    /// (`op`, `right`)
    Unary(UnaryOp, Box<Expr>),
    /// (`identifier`)
    Variable(Ident),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ident {
    pub symbol: String,
    pub span: Span,
}
impl Ident {
    pub fn new(symbol: String, span: Span) -> Self {
        Self { symbol, span }
    }

    pub fn from_token(token: Token) -> Self {
        Self {
            symbol: token.lexeme,
            span: token.span,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Plus,
    Minus,
    Multiply,
    Divide,
}
impl From<TokenKind> for BinaryOp {
    fn from(value: TokenKind) -> Self {
        match value {
            TokenKind::EqualEqual => Self::Equal,
            TokenKind::BangEqual => Self::NotEqual,
            TokenKind::Greater => Self::Greater,
            TokenKind::GreaterEqual => Self::GreaterEqual,
            TokenKind::Less => Self::Less,
            TokenKind::LessEqual => Self::LessEqual,
            TokenKind::Plus => Self::Plus,
            TokenKind::Minus => Self::Minus,
            TokenKind::Star => Self::Multiply,
            TokenKind::Slash => Self::Divide,
            _ => unreachable!(),
        }
    }
}
impl From<Token> for BinaryOp {
    fn from(value: Token) -> Self {
        Self::from(value.kind)
    }
}
impl BinaryOp {
    pub fn as_str(&self) -> &str {
        match self {
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::Greater => ">",
            BinaryOp::GreaterEqual => ">=",
            BinaryOp::Less => "<",
            BinaryOp::LessEqual => "<=",
            BinaryOp::Plus => "+",
            BinaryOp::Minus => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Negative,
    Not,
}
impl From<TokenKind> for UnaryOp {
    fn from(value: TokenKind) -> Self {
        match value {
            TokenKind::Bang => Self::Not,
            TokenKind::Minus => Self::Negative,
            _ => unreachable!(),
        }
    }
}
impl From<Token> for UnaryOp {
    fn from(value: Token) -> Self {
        Self::from(value.kind)
    }
}
impl UnaryOp {
    pub fn as_str(&self) -> &str {
        match self {
            UnaryOp::Negative => "-",
            UnaryOp::Not => "!",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Expr {
    id: usize,
    pub kind: ExprKind,
    pub span: Span,
}
impl Hash for Expr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Expr {}
impl Expr {
    pub fn new(kind: ExprKind, span: Span) -> Self {
        let id = EXPR_ID.fetch_add(1, Ordering::SeqCst);
        Self { id, kind, span }
    }

    pub fn assign(var: Ident, ex: Expr) -> Self {
        let span = var.span.to(ex.span);
        Self::new(ExprKind::Assign(var, Box::new(ex)), span)
    }

    pub fn binary(left: Expr, op: Token, right: Expr) -> Self {
        let span = left.span.to(right.span);
        Self::new(
            ExprKind::Binary(Box::new(left), op.into(), Box::new(right)),
            span,
        )
    }

    pub fn call(callee: Expr, arg_span: Span, args: Vec<Expr>) -> Self {
        Self::new(ExprKind::Call(Box::new(callee), arg_span, args), arg_span)
    }

    pub fn grouping(ex: Expr) -> Self {
        Self::new(ExprKind::Grouping(Box::new(ex.to_owned())), ex.span)
    }

    pub fn literal_string(str: String, span: Span) -> Self {
        Self::new(ExprKind::Literal(Literal::String(str)), span)
    }

    pub fn literal_number(num: f64, span: Span) -> Self {
        Self::new(ExprKind::Literal(Literal::Number(num)), span)
    }

    pub fn literal_bool(b: bool, span: Span) -> Self {
        Self::new(ExprKind::Literal(Literal::Bool(b)), span)
    }

    pub fn literal_null(span: Span) -> Self {
        Self::new(ExprKind::Literal(Literal::Null), span)
    }

    pub fn logical(left: Expr, op: Token, right: Expr) -> Self {
        let span = left.span.to(right.span);
        Self::new(
            ExprKind::Logical(Box::new(left), op.to_owned(), Box::new(right)),
            span,
        )
    }

    pub fn unary(op: Token, ex: Expr) -> Self {
        let span = op.span.to(ex.span);
        Self::new(ExprKind::Unary(UnaryOp::from(op), Box::new(ex)), span)
    }

    pub fn var(var: Token) -> Self {
        let span = var.span;
        Self::new(ExprKind::Variable(Ident::from_token(var)), span)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}
impl Hash for Literal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Literal::Number(num) => num.to_ne_bytes().hash(state),
            Literal::String(val) => val.hash(state),
            Literal::Bool(val) => val.hash(state),
            Literal::Null => mem::discriminant(self).hash(state),
        }
    }
}
impl Literal {
    pub fn as_str(&self) -> String {
        match self {
            Literal::String(str) => str.to_owned(),
            Literal::Number(num) => num.to_string(),
            Literal::Bool(lit) => lit.to_string(),
            Literal::Null => String::from("null"),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Literal::Bool(b) => *b,
            Literal::Null => false,
            _ => true,
        }
    }
}
