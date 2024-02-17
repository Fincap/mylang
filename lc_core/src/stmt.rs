use crate::{expr::ExprKind, token::Token};

#[derive(Clone, Debug)]
pub enum Stmt {
    /// (`statements`)
    Block(Vec<Stmt>),
    /// (`expression`)
    Expression(ExprKind),
    /// (`identifier`, `params`, `body`)
    Function(Token, Vec<Token>, Vec<Stmt>),
    /// (`condition`, `then`, `else`)
    If(ExprKind, Box<Stmt>, Option<Box<Stmt>>),
    /// (`expression`)
    Print(ExprKind),
    /// (`expression`)
    Return(ExprKind),
    /// (`identifier`, `initializer`)
    Let(Token, ExprKind),
    /// (`condition`, `body`)
    While(ExprKind, Box<Stmt>),
}
impl Stmt {
    pub fn new_if(ex: ExprKind, st_then: Stmt, st_else: Option<Stmt>) -> Self {
        let o_else = match st_else {
            Some(stmt) => Some(Box::new(stmt)),
            None => None,
        };
        Self::If(ex, Box::new(st_then), o_else)
    }

    pub fn new_while(ex: ExprKind, stmt: Stmt) -> Self {
        Self::While(ex, Box::new(stmt))
    }
}
