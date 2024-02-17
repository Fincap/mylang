use crate::token::Token;

pub const LIMIT_FN_ARGS: usize = 255;

#[derive(Clone, Debug)]
pub enum ExprKind {
    /// (`identifier`, `initializer`)
    Assign(Token, Box<ExprKind>),
    /// (`left`, `op`, `right`)
    Binary(Box<ExprKind>, Token, Box<ExprKind>),
    /// (`callee`, `paren`, `args`)
    Call(Box<ExprKind>, Token, Vec<ExprKind>),
    /// (`expression`)
    Grouping(Box<ExprKind>),
    /// (`literal`)
    Literal(Literal),
    /// (`left`, `op`, `right`)
    Logical(Box<ExprKind>, Token, Box<ExprKind>),
    /// (`op`, `right`)
    Unary(Token, Box<ExprKind>),
    /// (`identifier`)
    Variable(Token),
}
impl ExprKind {
    pub fn assign(var: Token, ex: ExprKind) -> Self {
        Self::Assign(var, Box::new(ex))
    }

    pub fn binary(left: ExprKind, op: Token, right: ExprKind) -> Self {
        Self::Binary(Box::new(left), op, Box::new(right))
    }

    pub fn call(callee: ExprKind, paren: Token, args: Vec<ExprKind>) -> Self {
        Self::Call(Box::new(callee), paren, args)
    }

    pub fn grouping(ex: ExprKind) -> Self {
        Self::Grouping(Box::new(ex))
    }

    pub fn literal_string(str: String) -> Self {
        Self::Literal(Literal::String(str))
    }

    pub fn literal_number(num: f64) -> Self {
        Self::Literal(Literal::Number(num))
    }

    pub fn literal_bool(b: bool) -> Self {
        Self::Literal(Literal::Bool(b))
    }

    pub fn literal_null() -> Self {
        Self::Literal(Literal::Null)
    }

    pub fn logical(left: ExprKind, op: Token, right: ExprKind) -> Self {
        Self::Logical(Box::new(left), op, Box::new(right))
    }

    pub fn unary(op: Token, ex: ExprKind) -> Self {
        Self::Unary(op, Box::new(ex))
    }

    pub fn var(var: Token) -> Self {
        Self::Variable(var)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
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
