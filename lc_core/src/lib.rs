mod error;
mod expr;
mod lexer;
mod parser;
mod stmt;
mod token;

pub use crate::error::*;
pub use crate::expr::*;
pub use crate::lexer::*;
pub use crate::parser::*;
pub use crate::stmt::*;
pub use crate::token::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
