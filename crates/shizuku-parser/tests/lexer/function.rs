use shizuku_parser::Lexer;
use shizuku_parser::Token;

#[test]
fn test_function() {
    let source = r#"
    fn sum(arg1: i32, arg2: i32) -> i32 {
        let sum = arg1 + arg2;
        return sum;
    }
    "#;
    let chars = source.char_indices().map(|(i, c)| (i as u32, c));
    let mut lexer = Lexer::new(chars);

    let expected_tokens = vec![
        (0, Token::NewLine, 1), // First newline from initial empty line
        (5, Token::Fn, 7),
        (8, Token::Ident { name: "sum".into() }, 11),
        (11, Token::LParen, 12),
        (
            12,
            Token::Ident {
                name: "arg1".into(),
            },
            16,
        ),
        (16, Token::Colon, 17),
        (18, Token::Ident { name: "i32".into() }, 21),
        (21, Token::Comma, 22),
        (
            23,
            Token::Ident {
                name: "arg2".into(),
            },
            27,
        ),
        (27, Token::Colon, 28),
        (29, Token::Ident { name: "i32".into() }, 32),
        (32, Token::RParen, 33),
        (34, Token::MinusRArrow, 36),
        (37, Token::Ident { name: "i32".into() }, 40),
        (41, Token::LBrace, 42),
        (42, Token::NewLine, 43), // Newline after {
        (51, Token::Let, 54),
        (55, Token::Ident { name: "sum".into() }, 58),
        (59, Token::Equal, 60),
        (
            61,
            Token::Ident {
                name: "arg1".into(),
            },
            65,
        ),
        (66, Token::Plus, 67),
        (
            68,
            Token::Ident {
                name: "arg2".into(),
            },
            72,
        ),
        (72, Token::Semicolon, 73),
        (73, Token::NewLine, 74), // Newline after let statement
        (82, Token::Return, 88),
        (89, Token::Ident { name: "sum".into() }, 92),
        (92, Token::Semicolon, 93),
        (93, Token::NewLine, 94), // Newline after return statement
        (98, Token::RBrace, 99),
        (99, Token::NewLine, 100), // Newline after }
        (104, Token::EOF, 104),    // EOF at end of input
    ];

    for (start, token, end) in expected_tokens {
        let actual = lexer.next().unwrap();
        assert_eq!(actual, (start, token, end));
    }
}
