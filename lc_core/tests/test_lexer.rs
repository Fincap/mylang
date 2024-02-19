use lc_core::*;
use TokenKind::*;

fn assert_lexer_tokens(source: &'static str, output: Vec<TokenKind>, len: usize) -> Vec<Token> {
    let tokens = Scanner::new(source.to_string()).scan_tokens();
    dbg!(&tokens);
    assert_eq!(tokens.len(), len);
    for (t, o) in tokens.iter().zip(output.iter()) {
        assert_eq!(t.kind, *o);
    }
    tokens
}

#[test]
fn scanner_empty() {
    assert_lexer_tokens("", vec![EOF], 1);
}

#[test]
fn scanner_keywords() {
    assert_lexer_tokens(
        "and class else false fn for if let null or 
        print return super this true while;",
        vec![
            And, Class, Else, False, Fn, For, If, Let, Null, Or, Print, Return, Super, This, True,
            While, Semicolon, EOF,
        ],
        18,
    );
}

#[test]
fn scanner_tokens() {
    assert_lexer_tokens(
        "(){}{{},.;-+/!<>*)=",
        vec![
            LeftParen, RightParen, LeftBrace, RightBrace, LeftBrace, LeftBrace, RightBrace, Comma,
            Dot, Semicolon, Minus, Plus, Slash, Bang, Less, Greater, Star, RightParen, Equal, EOF,
        ],
        20,
    );
    assert_lexer_tokens(
        "---++!=<<=+=+-=/=**=/>>=>",
        vec![
            MinusMinus,
            Minus,
            PlusPlus,
            BangEqual,
            Less,
            LessEqual,
            PlusEqual,
            Plus,
            MinusEqual,
            SlashEqual,
            Star,
            StarEqual,
            Slash,
            Greater,
            GreaterEqual,
            Greater,
            EOF,
        ],
        17,
    );
}

#[test]
fn scanner_comments() {
    assert_lexer_tokens(
        "let x = 0; // comment begins; x++;",
        vec![Let, Identifier, Equal, Number(0.0), Semicolon, EOF],
        6,
    );
    assert_lexer_tokens(
        "for (;;) //infinite loop
        {
            /* processing begins
            ...
            and then it ends */
            print null;
        }",
        vec![
            For, LeftParen, Semicolon, Semicolon, RightParen, LeftBrace, Print, Null, Semicolon,
            RightBrace, EOF,
        ],
        11,
    );
    assert_lexer_tokens(
        "while (true /* infinite loop */) print \"hello world!\"",
        vec![
            While,
            LeftParen,
            True,
            RightParen,
            Print,
            String("hello world!".into()),
            EOF,
        ],
        7,
    );
}

#[test]
fn scanner_literals() {
    assert_lexer_tokens(
        "x = 13 = \"string\"\"another string\";3.14159",
        vec![
            Identifier,
            Equal,
            Number(13.0),
            Equal,
            String("string".into()),
            String("another string".into()),
            Semicolon,
            Number(3.14159),
            EOF,
        ],
        9,
    );
}

#[test]
fn scanner_line_numbers() {
    let source = "6, 7;
    newline,
    
    \"spaced_gap\"
    /* multiline
    comment
    
    */
    return"
        .into();
    let output = vec![
        Number(6.0),
        Comma,
        Number(7.0),
        Semicolon,
        Identifier,
        Comma,
        String("spaced_gap".into()),
        Return,
        EOF,
    ];
    let expected_lines = vec![1, 1, 1, 1, 2, 2, 4, 9, 9];
    let tokens = assert_lexer_tokens(source, output, 9);
    for (t, l) in tokens.iter().zip(expected_lines.iter()) {
        assert_eq!(t.line, *l);
    }
}

#[test]
fn scanner_unterminated_string() {
    assert_lexer_tokens(
        "\"terminated\"identifer",
        vec![String("terminated".into()), Identifier, EOF],
        3,
    );
    assert_lexer_tokens(
        "\"terminated\"\"unterminated",
        vec![String("terminated".into()), EOF],
        2,
    );
    assert_lexer_tokens("\"unterminated", vec![EOF], 1);
    assert_lexer_tokens("\"\"", vec![String("".into()), EOF], 2);
}

#[test]
fn scanner_identifiers() {
    assert_lexer_tokens(
        "valid; _alsoValid; 12ident; id_valid6",
        vec![
            Identifier,
            Semicolon,
            Identifier,
            Semicolon,
            Number(12.0),
            Identifier,
            Semicolon,
            Identifier,
            EOF,
        ],
        9,
    );
}

#[test]
fn scanner_invalid() {
    assert_lexer_tokens(
        "@test;$let?;256%8'ident'~\"#lc@email.au\"",
        vec![
            Identifier,
            Semicolon,
            Let,
            Semicolon,
            Number(256.0),
            Number(8.0),
            Identifier,
            String("#lc@email.au".into()),
            EOF,
        ],
        9,
    );
}
