mod common;

use crate::common::t_hash;
use lc_core::*;
use TokenKind::*;

#[test]
fn token_hash() {
    let a = Token::new(Null, "null".into(), Span::new(1));
    let b = Token::new(Null, "null".into(), Span::new(1));
    let c = Token::new(If, "null".into(), Span::new(1));
    let d = Token::new(Null, "null".into(), Span::new(2));
    let e = Token::new(Null, "null ".into(), Span::new(2));

    assert_eq!(t_hash(&a), t_hash(b));
    assert_eq!(t_hash(&a), t_hash(c));
    assert_ne!(t_hash(&a), t_hash(&d));
    assert_ne!(t_hash(&d), t_hash(e));
}

#[test]
fn token_eq() {
    let a = Token::new(Null, "null".into(), Span::new(1));
    let b = Token::new(Null, "null".into(), Span::new(1));
    let c = Token::new(If, "null".into(), Span::new(1));
    let d = Token::new(Null, "null".into(), Span::new(2));
    let e = Token::new(Null, "null ".into(), Span::new(2));

    assert_eq!(a, b);
    assert_eq!(a, c);
    assert_ne!(a, d);
    assert_ne!(d, e);
}
