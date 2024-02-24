use core::fmt;
use std::{ops, sync::Mutex};

use once_cell::sync::Lazy;
use string_interner::{DefaultStringInterner, StringInterner};

use crate::Literal;

static STRING_TABLE: Lazy<Mutex<DefaultStringInterner>> =
    Lazy::new(|| Mutex::new(StringInterner::default()));

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Symbol {
    symbol: string_interner::DefaultSymbol,
}
impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            STRING_TABLE.lock().unwrap().resolve(self.symbol).unwrap()
        )
    }
}
impl ops::Add for Symbol {
    type Output = Symbol;

    fn add(self, rhs: Self) -> Self::Output {
        let lhs = STRING_TABLE
            .lock()
            .unwrap()
            .resolve(self.symbol)
            .unwrap()
            .to_owned();
        let rhs = STRING_TABLE
            .lock()
            .unwrap()
            .resolve(rhs.symbol)
            .unwrap()
            .to_owned();
        Symbol::new(&[lhs, rhs].join(""))
    }
}
impl Symbol {
    pub fn new(string: &str) -> Self {
        Self {
            symbol: STRING_TABLE.lock().unwrap().get_or_intern(string),
        }
    }

    pub fn as_lit(&self) -> Literal {
        Literal::String(*self)
    }
}
