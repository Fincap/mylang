use crate::token::Token;

pub const LIMIT_FN_ARGS: usize = 255;

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
}
impl Expr {
    pub fn assign(var: Token, ex: Expr) -> Self {
        Self {
            kind: ExprKind::Assign(var, Box::new(ex)),
        }
    }

    pub fn binary(left: Expr, op: Token, right: Expr) -> Self {
        Self {
            kind: ExprKind::Binary(Box::new(left), op, Box::new(right)),
        }
    }

    pub fn call(callee: Expr, paren: Token, args: Vec<Expr>) -> Self {
        Self {
            kind: ExprKind::Call(Box::new(callee), paren, args),
        }
    }

    pub fn grouping(ex: Expr) -> Self {
        Self {
            kind: ExprKind::Grouping(Box::new(ex)),
        }
    }

    pub fn literal_string(str: String) -> Self {
        Self {
            kind: ExprKind::Literal(Literal::String(str)),
        }
    }

    pub fn literal_number(num: f64) -> Self {
        Self {
            kind: ExprKind::Literal(Literal::Number(num)),
        }
    }

    pub fn literal_bool(b: bool) -> Self {
        Self {
            kind: ExprKind::Literal(Literal::Bool(b)),
        }
    }

    pub fn literal_null() -> Self {
        Self {
            kind: ExprKind::Literal(Literal::Null),
        }
    }

    pub fn logical(left: Expr, op: Token, right: Expr) -> Self {
        Self {
            kind: ExprKind::Logical(Box::new(left), op, Box::new(right)),
        }
    }

    pub fn unary(op: Token, ex: Expr) -> Self {
        Self {
            kind: ExprKind::Unary(op, Box::new(ex)),
        }
    }

    pub fn var(var: Token) -> Self {
        Self {
            kind: ExprKind::Variable(var),
        }
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
