#![allow(non_snake_case)]
use shizuku_parser::Lexer;
use shizuku_parser::LexicalError;
use shizuku_parser::LexicalErrorType::*;
use shizuku_parser::NumberBase as Base;
use shizuku_parser::SrcSpan;
use shizuku_parser::Token;

macro_rules! generate_valid_number_tests {
        ($($name:ident: $input:expr => $expected:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let chars = $input.char_indices().map(|(i, c)| (i as u32, c));
                    let mut lexer = Lexer::new(chars);

                    let token = lexer.next().unwrap();
                    assert_eq!(token, $expected);
                }
            )*
        };
    }

macro_rules! generate_invalid_number_tests{
        ($($name:ident: $input:expr => $expected:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let chars = $input.char_indices().map(|(i, c)| (i as u32, c));
                    let mut lexer = Lexer::new(chars);

                    let token = lexer.next().unwrap_err();
                    assert_eq!(token, $expected);
                }
            )*
        };
    }

// valid integer and float
generate_valid_number_tests! {
    test_number_3_14: "3.14" => (
        0,
        Token::Float {
            has_exp: false,
            value: "3.14".into(),
        },
        4,
    ),
    test_number_0_5: ".5" => (
        0,
        Token::Float {
            has_exp: false,
            value: ".5".into(),
        },
        2,
    ),
    test_number_10: "10." => (
        0,
        Token::Float {
            has_exp: false,
            value: "10.".into(),
        },
        3,
    ),
    test_number_1e10: "1e10" => (
        0,
        Token::Float {
            has_exp: true,
            value: "1e10".into(),
        },
        4,
    ),
    test_number_2_9e_minus_3: "2.9e-3" => (
        0,
        Token::Float {
            has_exp: true,
            value: "2.9e-3".into(),
        },
        6,
    ),
    test_number_3E_plus_4: "3E+4" => (
        0,
        Token::Float {
            has_exp: true,
            value: "3E+4".into(),
        },
        4,
    ),
    test_number_0_0: "0.0" => (
        0,
        Token::Float {
            has_exp: false,
            value: "0.0".into(),
        },
        3,
    ),
    test_number_minus_0: "-0" => (
        0,
        Token::Int {
            base: Base::Decimal,
            value: "-0".into(),
        },
        2,
    ),
    test_number_plus_0: "+0" => (
        0,
        Token::Int{
            base: Base::Decimal,
            value: "+0".into(),
        },
        2,
    ),
    test_number_0_2: "0.2" => (
        0,
        Token::Float {
            has_exp: false,
            value: "0.2".into(),
        },
        3,
    ),
    test_number_2_123456: "2.123456" => (
        0,
        Token::Float {
            has_exp: false,
            value: "2.123456".into(),
        },
        8,
    ),
    test_number_dot_2: ".2" => (
        0,
        Token::Float {
            has_exp: false,
            value: ".2".into(),
        },
        2,
    ),
    test_number_2_dot: "2." => (
        0,
        Token::Float {
            has_exp: false,
            value: "2.".into(),
        },
        2,
    ),
    test_number_minus_2_5: "-2.5" => (
        0,
        Token::Float {
            has_exp: false,
            value: "-2.5".into(),
        },
        4,
    ),
    test_number_plus_2_5: "+2.5" => (
        0,
        Token::Float {
            has_exp: false,
            value: "+2.5".into(),
        },
        4,
    ),
    test_number_1e3: "1e3" => (
        0,
        Token::Float {
            has_exp: true,
            value: "1e3".into(),
        },
        3,
    ),
    test_number_dot_1e3: ".1e3" => (
        0,
        Token::Float {
            has_exp: true,
            value: ".1e3".into(),
        },
        4,
    ),
    test_number_1e_plus_3: "1e+3" => (
        0,
        Token::Float {
            has_exp: true,
            value: "1e+3".into(),
        },
        4,
    ),
    test_number_1e_minus_3: "1e-3" => (
        0,
        Token::Float {
            has_exp: true,
            value: "1e-3".into(),
        },
        4,
    ),
    test_number_minus_1e_minus_3: "-1e-3" => (
        0,
        Token::Float {
            has_exp: true,
            value: "-1e-3".into(),
        },
        5,
    ),
    test_number_plus_1e3: "+1e3" => (
        0,
        Token::Float {
            has_exp: true,
            value: "+1e3".into(),
        },
        4,
    ),
    test_number_0e0: "0e0" => (
        0,
        Token::Float {
            has_exp: true,
            value: "0e0".into(),
        },
        3,
    ),
    test_number_minus_0e0: "-0e0" => (
        0,
        Token::Float {
            has_exp: true,
            value: "-0e0".into(),
        },
        4,
    ),
    test_number_plus_0e0: "+0e0" => (
        0,
        Token::Float {
            has_exp: true,
            value: "+0e0".into(),
        },
        4,
    ),
    test_number_123_456: "123.456" => (
        0,
        Token::Float {
            has_exp: false,
            value: "123.456".into(),
        },
        7,
    ),
    test_number_1e1000: "1e1000" => (
        0,
        Token::Float {
            has_exp: true,
            value: "1e1000".into(),
        },
        6,
    ),
    test_number_1e_minus_1000: "1e-1000" => (
        0,
        Token::Float {
            has_exp: true,
            value: "1e-1000".into(),
        },
        7,
    ),
    test_number_1e_plus_1000: "1e+1000" => (
        0,
        Token::Float {
            has_exp: true,
            value: "1e+1000".into(),
        },
        7,
    ),
    test_number_1_000_000_1: "1_000.000_1" => (
        0,
        Token::Float {
            has_exp: false,
            value: "1_000.000_1".into(),
        },
        11,
    ),
    test_number_minus_1_dot: "-1." => (
        0,
        Token::Float {
            has_exp: false,
            value: "-1.".into(),
        },
        3,
    ),
    test_number_plus_1_dot: "+1." => (
        0,
        Token::Float {
            has_exp: false,
            value: "+1.".into(),
        },
        3,
    ),
    test_number_00: "00" => (
        0,
        Token::Int {
            base:Base::Decimal,
            value: "00".into(),
        },
        2,
    ),
}

// invalid integer and float
generate_invalid_number_tests! {
    test_number_1_: "1_" => (
        LexicalError { error: IllegalLiteral {  tok: '_' }, location: SrcSpan { start: 0, end: 2 } }
    ),
    test_number_0e: "0e" => (
        LexicalError { error: IllegalLiteral {  tok: 'e' }, location: SrcSpan { start: 0, end: 2 } }
    ),
    test_number_07: "07" => (
        LexicalError { error: IllegalLiteral {  tok: '7' }, location: SrcSpan { start: 0, end: 2 } }
    ),
    test_number_001: "001" => (
        LexicalError { error: IllegalLiteral {  tok: '1' }, location: SrcSpan { start: 0, end: 3 } }
    ),
    test_number_0e_3: "0e_3" => (
        LexicalError { error: IllegalLiteral {  tok: '_' }, location: SrcSpan { start: 0, end: 3 } }
    ),
    test_number_0_3: "0_3" => (
        LexicalError { error: IllegalLiteral {  tok: '_' }, location: SrcSpan { start: 0, end: 2 } }
    ),
    test_number_1__3: "1__3" => (
        LexicalError { error: IllegalLiteral {  tok: '_' }, location: SrcSpan { start: 0, end: 3 } }
    ),
    test_number_0_x3: "0_x3" => (
        LexicalError { error: IllegalLiteral {  tok: '_' }, location: SrcSpan { start: 0, end: 2 } }
    ),
}

#[test]
fn test_int_chunk() {
    let source = "32_64 0b10 0xFF 0o7 0";
    let chars = source.char_indices().map(|(i, c)| (i as u32, c));
    let mut lexer = Lexer::new(chars);

    let expected_tokens = vec![
        (
            0,
            Token::Int {
                base: Base::Decimal,
                value: "32_64".into(),
            },
            5,
        ),
        (
            6,
            Token::Int {
                base: Base::Binary,
                value: "0b10".into(),
            },
            10,
        ),
        (
            11,
            Token::Int {
                base: Base::Hexadecimal,
                value: "0xFF".into(),
            },
            15,
        ),
        (
            16,
            Token::Int {
                base: Base::Octal,
                value: "0o7".into(),
            },
            19,
        ),
        (
            20,
            Token::Int {
                base: Base::Decimal,
                value: "0".into(),
            },
            21,
        ),
    ];

    for (start, expected_token, end) in expected_tokens {
        let token = lexer.next().unwrap();
        assert_eq!(token, (start, expected_token, end));
    }
}

#[test]
fn test_float_chunk() {
    let source = "3.14 .5 10. 1e10 2.9e-3 3E+4 0.0 -0 +0 0.2 2.123456 .2 2. -2.5 +2.5 1e3 1e+3 1e-3 -1e-3 +1e3 0e0 -0e0 +0e0 123.456 1e1000 1e-1000 1e+1000 1_000.000_1 -1. +1.";
    let chars = source.char_indices().map(|(i, c)| (i as u32, c));
    let mut lexer = Lexer::new(chars);

    let expected_tokens = vec![
        // Original test cases
        (
            0,
            Token::Float {
                has_exp: false,
                value: "3.14".into(),
            },
            4,
        ),
        (
            5,
            Token::Float {
                has_exp: false,
                value: ".5".into(),
            },
            7,
        ),
        (
            8,
            Token::Float {
                has_exp: false,
                value: "10.".into(),
            },
            11,
        ),
        (
            12,
            Token::Float {
                has_exp: true,
                value: "1e10".into(),
            },
            16,
        ),
        (
            17,
            Token::Float {
                has_exp: true,
                value: "2.9e-3".into(),
            },
            23,
        ),
        (
            24,
            Token::Float {
                has_exp: true,
                value: "3E+4".into(),
            },
            28,
        ),
        // New test cases
        (
            29,
            Token::Float {
                has_exp: false,
                value: "0.0".into(),
            },
            32,
        ),
        (
            33,
            Token::Int {
                base: Base::Decimal,
                value: "-0".into(),
            },
            35,
        ),
        (
            36,
            Token::Int {
                base: Base::Decimal,
                value: "+0".into(),
            },
            38,
        ),
        (
            39,
            Token::Float {
                has_exp: false,
                value: "0.2".into(),
            },
            42,
        ),
        (
            43,
            Token::Float {
                has_exp: false,
                value: "2.123456".into(),
            },
            51,
        ),
        (
            52,
            Token::Float {
                has_exp: false,
                value: ".2".into(),
            },
            54,
        ),
        (
            55,
            Token::Float {
                has_exp: false,
                value: "2.".into(),
            },
            57,
        ),
        (
            58,
            Token::Float {
                has_exp: false,
                value: "-2.5".into(),
            },
            62,
        ),
        (
            63,
            Token::Float {
                has_exp: false,
                value: "+2.5".into(),
            },
            67,
        ),
        (
            68,
            Token::Float {
                has_exp: true,
                value: "1e3".into(),
            },
            71,
        ),
        (
            72,
            Token::Float {
                has_exp: true,
                value: "1e+3".into(),
            },
            76,
        ),
        (
            77,
            Token::Float {
                has_exp: true,
                value: "1e-3".into(),
            },
            81,
        ),
        (
            82,
            Token::Float {
                has_exp: true,
                value: "-1e-3".into(),
            },
            87,
        ),
        (
            88,
            Token::Float {
                has_exp: true,
                value: "+1e3".into(),
            },
            92,
        ),
        (
            93,
            Token::Float {
                has_exp: true,
                value: "0e0".into(),
            },
            96,
        ),
        (
            97,
            Token::Float {
                has_exp: true,
                value: "-0e0".into(),
            },
            101,
        ),
        (
            102,
            Token::Float {
                has_exp: true,
                value: "+0e0".into(),
            },
            106,
        ),
        (
            107,
            Token::Float {
                has_exp: false,
                value: "123.456".into(),
            },
            114,
        ),
        (
            115,
            Token::Float {
                has_exp: true,
                value: "1e1000".into(),
            },
            121,
        ),
        (
            122,
            Token::Float {
                has_exp: true,
                value: "1e-1000".into(),
            },
            129,
        ),
        (
            130,
            Token::Float {
                has_exp: true,
                value: "1e+1000".into(),
            },
            137,
        ),
        (
            138,
            Token::Float {
                has_exp: false,
                value: "1_000.000_1".into(),
            },
            149,
        ),
        (
            150,
            Token::Float {
                has_exp: false,
                value: "-1.".into(),
            },
            153,
        ),
        (
            154,
            Token::Float {
                has_exp: false,
                value: "+1.".into(),
            },
            157,
        ),
    ];

    for (start, expected_token, end) in expected_tokens {
        let token = lexer.next().unwrap();
        assert_eq!(token, (start, expected_token, end));
    }
}
