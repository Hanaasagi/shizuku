use shizuku_parser::Lexer;
use shizuku_parser::Token;

#[test]
fn test_struct_define() {
    let source = r#"
    struct MyStruct {
        field1: i32,
        field2: i64
    }
    "#;
    let chars = source.char_indices().map(|(i, c)| (i as u32, c));
    let mut lexer = Lexer::new(chars);

    let expected_tokens = vec![
        (0, Token::NewLine, 1), // First newline from initial empty line
        (5, Token::Struct, 11),
        (
            12,
            Token::Ident {
                name: "MyStruct".into(),
            },
            20,
        ),
        (21, Token::LBrace, 22),
        (22, Token::NewLine, 23), // Newline after {
        (
            31,
            Token::Ident {
                name: "field1".into(),
            },
            37,
        ),
        (37, Token::Colon, 38),
        (39, Token::Ident { name: "i32".into() }, 42),
        (42, Token::Comma, 43),
        (43, Token::NewLine, 44), // Newline after field1
        (
            52,
            Token::Ident {
                name: "field2".into(),
            },
            58,
        ),
        (58, Token::Colon, 59),
        (60, Token::Ident { name: "i64".into() }, 63),
        (63, Token::NewLine, 64), // Newline after field2
        (68, Token::RBrace, 69),
        (69, Token::NewLine, 70), // Newline after }
        (74, Token::EOF, 74),     // EOF at end of input
    ];

    for (start, token, end) in expected_tokens {
        let actual = lexer.next().unwrap();
        assert_eq!(actual, (start, token, end));
    }
}
