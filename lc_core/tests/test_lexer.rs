use lc_core::*;
use TokenType::*;

fn assert_lexer_tokens(source: &'static str, output: Vec<TokenType>, len: usize) {
    let tokens = Scanner::new(source.to_string()).scan_tokens();
    assert_eq!(tokens.len(), len);
    for (t, o) in tokens.iter().zip(output.iter()) {
        assert_eq!(t.t_type, *o);
    }
}

#[test]
fn test_empty() {
    assert_lexer_tokens("", vec![EOF], 1);
}

#[test]
fn test_keywords() {
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
fn test_tokens() {
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
fn test_comments() {
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
            String("hello world!".to_string()),
            EOF,
        ],
        7,
    )
}

#[test]
fn test_literals() {
    assert_lexer_tokens(
        "x = 13 = \"string\"\"another string\";3.14159",
        vec![
            Identifier,
            Equal,
            Number(13.0),
            Equal,
            String("string".to_string()),
            String("another string".to_string()),
            Semicolon,
            Number(3.14159),
            EOF,
        ],
        9,
    )
}

#[test]
fn test_line_numbers() {
    let source = "6, 7;
    newline,
    
    \"spaced_gap\"
    /* multiline
    comment
    
    */
    return"
        .to_string();
    let output = vec![
        Number(6.0),
        Comma,
        Number(7.0),
        Semicolon,
        Identifier,
        Comma,
        String("spaced_gap".to_string()),
        Return,
        EOF,
    ];
    let expected_lines = vec![1, 1, 1, 1, 2, 2, 4, 9, 9];
    let tokens = Scanner::new(source.to_string()).scan_tokens();
    assert_eq!(tokens.len(), 9);
    for (t, o) in tokens.iter().zip(output.iter()) {
        assert_eq!(t.t_type, *o);
    }

    for (t, l) in tokens.iter().zip(expected_lines.iter()) {
        assert_eq!(t.line, *l);
    }
}
