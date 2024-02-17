use crate::{expr::Expr, token::Token};

#[derive(Clone, Debug)]
pub enum Stmt {
    /// (`statements`)
    Block(Vec<Stmt>),
    /// (`expression`)
    Expression(Expr),
    /// (`identifier`, `params`, `body`)
    Function(Token, Vec<Token>, Vec<Stmt>),
    /// (`condition`, `then`, `else`)
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    /// (`expression`)
    Print(Expr),
    /// (`expression`)
    Return(Expr),
    /// (`identifier`, `initializer`)
    Let(Token, Expr),
    /// (`condition`, `body`)
    While(Expr, Box<Stmt>),
}
impl Stmt {
    pub fn new_if(ex: Expr, st_then: Stmt, st_else: Option<Stmt>) -> Self {
        let o_else = match st_else {
            Some(stmt) => Some(Box::new(stmt)),
            None => None,
        };
        Self::If(ex, Box::new(st_then), o_else)
    }

    pub fn new_while(ex: Expr, stmt: Stmt) -> Self {
        Self::While(ex, Box::new(stmt))
    }
}
