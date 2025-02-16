use shizuku_parser::ASTNode;
use shizuku_parser::Parser;
use shizuku_parser::Token;
use shizuku_parser::ast::*;

#[test]
fn test_parse_function_declaration() {
    // fn add (a: i32, b: i32) -> i32 { return a + b; }
    let source_tokens = vec![
        (0, Token::Fn, 2),                             // fn
        (3, Token::Ident { name: "add".into() }, 6),   // add
        (6, Token::LParen, 7),                         // (
        (7, Token::Ident { name: "a".into() }, 8),     // a
        (8, Token::Colon, 9),                          // :
        (10, Token::Ident { name: "i32".into() }, 13), // i32
        (13, Token::Comma, 14),                        // ,
        (15, Token::Ident { name: "b".into() }, 16),   // b
        (16, Token::Colon, 17),                        // :
        (18, Token::Ident { name: "i32".into() }, 21), // i32
        (21, Token::RParen, 22),                       // )
        (23, Token::MinusRArrow, 25),                  // ->
        (26, Token::Ident { name: "i32".into() }, 29), // i32
        (30, Token::LBrace, 31),                       // {
        (32, Token::Return, 38),                       // return
        (39, Token::Ident { name: "a".into() }, 40),   // a
        (41, Token::Plus, 42),                         // +
        (43, Token::Ident { name: "b".into() }, 44),   // b
        (44, Token::Semicolon, 45),                    // ;
        (46, Token::RBrace, 47),                       // }
        (48, Token::EOF, 48),                          // EOF
    ];

    let mut parser = Parser::new(source_tokens.into_iter());
    let ast = parser.parse_program().expect("Failed to parse program");

    assert_eq!(ast, vec![ASTNode::Function {
        name: "add".into(),
        params: vec![
            Parameter {
                name: "a".into(),
                param_type: Type { name: "i32".into() },
            },
            Parameter {
                name: "b".into(),
                param_type: Type { name: "i32".into() },
            },
        ],
        return_type: Some(Type { name: "i32".into() }),
        body: vec![ASTNode::Return {
            value: Some(Box::new(ASTNode::BinaryOp {
                left: Box::new(ASTNode::Variable {
                    name: "a".into(),
                    value: None
                }),
                operator: Token::Plus,
                right: Box::new(ASTNode::Variable {
                    name: "b".into(),
                    value: None
                })
            })),
        }],
    }]);
}
