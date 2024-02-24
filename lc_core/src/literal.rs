use std::hash::{Hash, Hasher};
use std::{fmt, mem, ops};

use crate::RuntimeError;

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
impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::String(str) => write!(f, "{}", str),
            Literal::Number(num) => write!(f, "{}", num),
            Literal::Bool(lit) => write!(f, "{}", lit),
            Literal::Null => write!(f, "null"),
        }
    }
}
impl ops::Add for Literal {
    type Output = Result<Literal, RuntimeError>;

    fn add(self, rhs: Self) -> Self::Output {
        let err = Err(RuntimeError::new(
            "Operands must be two numbers or two strings.".into(),
        ));
        match self {
            Literal::Number(lhs) => match rhs {
                Literal::Number(rhs) => Ok(Literal::Number(lhs + rhs)),
                _ => err,
            },
            Literal::String(lhs) => match rhs {
                Literal::String(rhs) => Ok(Literal::String([lhs, rhs].join(""))),
                _ => err,
            },
            _ => err,
        }
    }
}
impl ops::Sub for Literal {
    type Output = Result<Literal, RuntimeError>;

    fn sub(self, rhs: Self) -> Self::Output {
        let err = Err(RuntimeError::new("Operands must be two numbers.".into()));
        match self {
            Literal::Number(lhs) => match rhs {
                Literal::Number(rhs) => Ok(Literal::Number(lhs - rhs)),
                _ => err,
            },
            _ => err,
        }
    }
}
impl ops::Mul for Literal {
    type Output = Result<Literal, RuntimeError>;

    fn mul(self, rhs: Self) -> Self::Output {
        let err = Err(RuntimeError::new("Operands must be two numbers.".into()));
        match self {
            Literal::Number(lhs) => match rhs {
                Literal::Number(rhs) => Ok(Literal::Number(lhs * rhs)),
                _ => err,
            },
            _ => err,
        }
    }
}
impl ops::Div for Literal {
    type Output = Result<Literal, RuntimeError>;

    fn div(self, rhs: Self) -> Self::Output {
        let err = Err(RuntimeError::new("Operands must be two numbers.".into()));
        match self {
            Literal::Number(lhs) => match rhs {
                Literal::Number(rhs) => Ok(Literal::Number(lhs / rhs)),
                _ => err,
            },
            _ => err,
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
