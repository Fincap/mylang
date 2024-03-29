use crate::{Expr, Ident};

#[derive(Clone, Debug)]
pub enum Stmt {
    /// (`statements`)
    Block(Vec<Stmt>),
    /// (`identifer`, `methods`)
    Class(Ident, Vec<Stmt>),
    /// (`expression`)
    Expression(Expr),
    /// (`identifier`, `params`, `body`)
    Function(Ident, Vec<Ident>, Vec<Stmt>),
    /// (`condition`, `then`, `else`)
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    /// (`expression`)
    Print(Expr),
    /// (`expression`)
    Return(Expr),
    /// (`identifier`, `initializer`)
    Let(Ident, Expr),
    /// (`condition`, `body`)
    While(Expr, Box<Stmt>),
}
impl Stmt {
    pub fn new_if(ex: Expr, st_then: Stmt, st_else: Option<Stmt>) -> Self {
        Self::If(ex, Box::new(st_then), st_else.map(Box::new))
    }

    pub fn new_while(ex: Expr, stmt: Stmt) -> Self {
        Self::While(ex, Box::new(stmt))
    }
}
