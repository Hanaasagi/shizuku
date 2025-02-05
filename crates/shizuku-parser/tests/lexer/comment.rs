#![allow(non_snake_case)]
use shizuku_parser::Lexer;
use shizuku_parser::Token;

#[test]
fn test_comment() {
    fn test_lexer_comment(source: &str, expected_token: (u32, Token, u32)) {
        let chars = source.char_indices().map(|(i, c)| (i as u32, c));
        let mut lexer = Lexer::new(chars);

        let token = lexer.next().unwrap();

        assert_eq!(token, expected_token);

        let start = token.0 as usize;
        let end = token.2 as usize;
        assert_eq!(&source[start..end], match &expected_token.1 {
            Token::Comment { content } => content.as_str(),
            _ => panic!("Expected a Comment token"),
        });
    }

    test_lexer_comment(
        "// This is Comment",
        (
            2,
            Token::Comment {
                content: " This is Comment".into(),
            },
            18,
        ),
    );

    test_lexer_comment(
        "    // This is Comment\n$",
        (
            6,
            Token::Comment {
                content: " This is Comment".into(),
            },
            22,
        ),
    );

    test_lexer_comment(
        "// This is \nComment",
        (
            2,
            Token::Comment {
                content: " This is ".into(),
            },
            11,
        ),
    );
}

#[test]
fn test_comment_doc() {
    fn test_lexer_comment_doc(source: &str, expected_token: (u32, Token, u32)) {
        let chars = source.char_indices().map(|(i, c)| (i as u32, c));
        let mut lexer = Lexer::new(chars);

        let token = lexer.next().unwrap();

        assert_eq!(token, expected_token);

        let start = token.0 as usize;
        let end = token.2 as usize;
        assert_eq!(&source[start..end], match &expected_token.1 {
            Token::CommentDoc { content } => content.as_str(),
            _ => panic!("Expected a DocComment token"),
        });
    }

    test_lexer_comment_doc(
        "/// This is Doc",
        (
            3,
            Token::CommentDoc {
                content: " This is Doc".into(),
            },
            15,
        ),
    );

    test_lexer_comment_doc(
        "    /// This is Doc\n$",
        (
            7,
            Token::CommentDoc {
                content: " This is Doc".into(),
            },
            19,
        ),
    );

    test_lexer_comment_doc(
        "/// This is \nDoc",
        (
            3,
            Token::CommentDoc {
                content: " This is ".into(),
            },
            12,
        ),
    );
}
