use std::hash::{Hash, Hasher};
use std::mem;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::token::Token;
use crate::TokenKind;

pub const LIMIT_FN_ARGS: usize = 255;
static EXPR_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum ExprKind {
    /// (`identifier`, `initializer`)
    Assign(Token, Box<Expr>),
    /// (`left`, `op`, `right`)
    Binary(Box<Expr>, Token, Box<Expr>),
    /// (`callee`, `paren`, `args`)
    Call(Box<Expr>, Token, Vec<Expr>),
    /// (`expression`)
    Grouping(Box<Expr>),
    /// (`literal`)
    Literal(Literal),
    /// (`left`, `op`, `right`)
    Logical(Box<Expr>, Token, Box<Expr>),
    /// (`op`, `right`)
    Unary(Token, Box<Expr>),
    /// (`identifier`)
    Variable(Token),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Neg,
    Not,
}
impl From<TokenKind> for UnaryOp {
    fn from(value: TokenKind) -> Self {
        match value {
            TokenKind::Bang => Self::Not,
            TokenKind::Minus => Self::Neg,
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
            UnaryOp::Neg => "-",
            UnaryOp::Not => "!",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Expr {
    id: usize,
    pub kind: ExprKind,
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
    pub fn new(kind: ExprKind) -> Self {
        let id = EXPR_ID.fetch_add(1, Ordering::SeqCst);
        Self { id, kind }
    }

    pub fn assign(var: Token, ex: Expr) -> Self {
        Self::new(ExprKind::Assign(var, Box::new(ex)))
    }

    pub fn binary(left: Expr, op: Token, right: Expr) -> Self {
        Self::new(ExprKind::Binary(Box::new(left), op, Box::new(right)))
    }

    pub fn call(callee: Expr, paren: Token, args: Vec<Expr>) -> Self {
        Self::new(ExprKind::Call(Box::new(callee), paren, args))
    }

    pub fn grouping(ex: Expr) -> Self {
        Self::new(ExprKind::Grouping(Box::new(ex)))
    }

    pub fn literal_string(str: String) -> Self {
        Self::new(ExprKind::Literal(Literal::String(str)))
    }

    pub fn literal_number(num: f64) -> Self {
        Self::new(ExprKind::Literal(Literal::Number(num)))
    }

    pub fn literal_bool(b: bool) -> Self {
        Self::new(ExprKind::Literal(Literal::Bool(b)))
    }

    pub fn literal_null() -> Self {
        Self::new(ExprKind::Literal(Literal::Null))
    }

    pub fn logical(left: Expr, op: Token, right: Expr) -> Self {
        Self::new(ExprKind::Logical(Box::new(left), op, Box::new(right)))
    }

    pub fn unary(op: Token, ex: Expr) -> Self {
        Self::new(ExprKind::Unary(op.into(), Box::new(ex)))
    }

    pub fn var(var: Token) -> Self {
        Self::new(ExprKind::Variable(var))
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
