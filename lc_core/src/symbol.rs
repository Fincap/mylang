use core::fmt;
use std::{hash, ops, sync::Mutex};

use once_cell::sync::Lazy;
use string_interner::{
    DefaultStringInterner, DefaultSymbol, StringInterner, Symbol as InternedSymbol,
};

use crate::Literal;

type InternTable = Lazy<Mutex<DefaultStringInterner>>;

static STRING_TABLE: InternTable = Lazy::new(|| Mutex::new(StringInterner::default()));
static IDENT_TABLE: InternTable = Lazy::new(|| Mutex::new(StringInterner::default()));

#[derive(Clone, Copy, Debug)]
pub struct Symbol {
    symbol: string_interner::DefaultSymbol,
    table: &'static InternTable,
}
impl hash::Hash for Symbol {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.symbol.hash(state);
        state.write_usize(self.table as *const InternTable as usize);
    }
}
impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.symbol == other.symbol
            && self.table as *const InternTable == other.table as *const InternTable
    }
}
impl Eq for Symbol {}
impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.symbol.partial_cmp(&other.symbol)
    }
}
impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.symbol.cmp(&other.symbol)
    }
}
impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.resolve(self.symbol))
    }
}
impl ops::Add for Symbol {
    type Output = Symbol;

    fn add(self, rhs: Self) -> Self::Output {
        let lhs = self.resolve(self.symbol);
        let rhs = self.resolve(rhs.symbol);
        Symbol::string(&[lhs, rhs].join(""))
    }
}
impl Symbol {
    pub fn string(string: &str) -> Self {
        Self {
            symbol: STRING_TABLE.lock().unwrap().get_or_intern(string),
            table: &STRING_TABLE,
        }
    }

    pub fn ident(string: &str) -> Self {
        Self {
            symbol: IDENT_TABLE.lock().unwrap().get_or_intern(string),
            table: &IDENT_TABLE,
        }
    }

    pub fn as_lit(&self) -> Literal {
        Literal::String(*self)
    }

    pub fn index(&self) -> usize {
        self.symbol.to_usize()
    }

    fn resolve(&self, symbol: DefaultSymbol) -> String {
        self.table
            .lock()
            .unwrap()
            .resolve(symbol)
            .unwrap()
            .to_owned()
    }
}
