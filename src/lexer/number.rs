use super::{lexer::*, utils::is_whitespace};
use crate::token::Base;
use crate::token::Token;
use ecow::EcoString;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Start,
    Sign,
    Zero,
    Int,
    LDot,
    Dot,
    Frac,
    Exp,
    ExpSign,
    ExpInt,
    Hex,
    Oct,
    Bin,
    End,
    Error,
}

// START:
//     SIGN -> SIGN
//     DIGIT -> INT
//     "0" -> ZERO
//     "." -> L_DOT
//     ERROR -> ERROR

// SIGN:
//     DIGIT -> INT
//     "0" -> ZERO
//     "." -> L_DOT
//     ERROR -> ERROR

// ZERO:
//     "x" | "X" -> HEX
//     "o" | "O" -> OCT
//     "b" | "B" -> BIN
//     "." -> L_DOT
//     DIGIT -> INT  # (Warning: Invalid octal sequence, e.g., 09)
//     "e" | "E" -> EXP
//     END -> END

// INT:
//     DIGIT -> INT
//     "." -> DOT
//     "e" | "E" -> EXP
//     END -> END

// DOT:
//     DIGIT -> FRAC
//     ERROR -> ERROR

// FRAC:
//     DIGIT -> FRAC
//     "e" | "E" -> EXP
//     END -> END

// EXP:
//     "+" | "-" -> EXP_SIGN
//     DIGIT -> EXP_INT
//     ERROR -> ERROR

// EXP_SIGN:
//     DIGIT -> EXP_INT
//     ERROR -> ERROR

// EXP_INT:
//     DIGIT -> EXP_INT
//     END -> END

// HEX:
//     HEX_DIGIT -> HEX
//     END -> END

// OCT:
//     OCT_DIGIT -> OCT
//     END -> END

// BIN:
//     "0" | "1" -> BIN
//     END -> END

// END:
//     ERROR -> ERROR

// ERROR:
//     ERROR -> ERROR

fn state_transition(state: State, chr: char) -> State {
    let mut state = state;

    if !matches!(state, State::Error | State::End) && is_whitespace(chr) {
        return State::End;
    }

    // handle `_`
    if chr == '_' {
        return state;
    }

    match state {
        State::End => {
            // let end = self.get_pos();
            // Ok((start, Token::Int { base, value }, end))
        }
        State::Error => {
            // let end = self.get_pos();
        }
        State::Start => {
            if chr == '+' || chr == '-' {
                state = State::Sign;
            } else if chr == '0' {
                state = State::Zero;
            } else if chr.is_digit(10) {
                state = State::Int;
            } else if chr == '.' {
                state = State::LDot;
            } else {
                state = State::Error;
            }
        }
        State::Sign => {
            if chr == '0' {
                state = State::Zero;
            } else if chr.is_digit(10) {
                state = State::Int;
            } else if chr == '.' {
                state = State::LDot;
            } else {
                state = State::Error;
            }
        }
        State::Zero => {
            if chr == 'x' || chr == 'X' {
                state = State::Hex;
            } else if chr == 'o' || chr == 'O' {
                state = State::Oct;
            } else if chr == 'b' || chr == 'B' {
                state = State::Bin;
            } else if chr == '.' {
                state = State::LDot;
            } else if chr.is_digit(10) {
                state = State::Int;
            } else if chr == 'e' || chr == 'E' {
                state = State::Exp;
            } else {
                state = State::End;
            }
        }
        State::Int => {
            if chr.is_digit(10) {
                state = State::Int;
            } else if chr == '.' {
                state = State::Dot;
            } else if chr == 'e' || chr == 'E' {
                state = State::Exp;
            } else {
                state = State::End;
            }
        }
        State::LDot => {
            if chr.is_digit(10) {
                state = State::Frac;
            } else {
                state = State::Error;
            }
        }
        State::Dot => {
            if chr.is_digit(10) {
                state = State::Frac;
            } else {
                state = State::Error;
            }
        }
        State::Frac => {
            if chr.is_digit(10) {
                state = State::Frac;
            } else if chr == 'e' || chr == 'E' {
                state = State::Exp;
            } else {
                state = State::End;
            }
        }
        State::Exp => {
            if chr == '+' || chr == '-' {
                state = State::ExpSign;
            } else if chr.is_digit(10) {
                state = State::ExpInt;
            } else {
                state = State::Error;
            }
        }
        State::ExpSign => {
            if chr.is_digit(10) {
                state = State::ExpInt;
            } else {
                state = State::Error;
            }
        }
        State::ExpInt => {
            if chr.is_digit(10) {
                state = State::ExpInt;
            } else {
                state = State::End;
            }
        }
        State::Hex => {
            if chr.is_digit(10)
                || chr.to_ascii_lowercase() >= 'a' && chr.to_ascii_lowercase() <= 'f'
            {
                state = State::Hex;
            } else {
                state = State::End;
            }
        }
        State::Oct => {
            if chr >= '0' && chr <= '7' {
                state = State::Oct;
            } else {
                state = State::End;
            }
        }
        State::Bin => {
            if chr == '0' || chr == '1' {
                state = State::Bin;
            } else {
                state = State::End;
            }
        }
    }
    return state;
}

impl<I> Lexer<I>
where
    I: Iterator<Item = (LOC, char)>,
{
    pub(super) fn consume_number_like(&mut self) -> LexResult {
        // enum Kind {
        //     Binary,
        //     Octal,
        //     Decimal,
        //     Hexadecimal,
        //     Float,
        //     ScientificNotationFloat,
        // }

        // At least one char
        debug_assert!(self.chr0.is_some());

        let mut state = State::Start;

        let mut value = EcoString::new();
        let start = self.get_pos();

        let mut new_state;

        loop {
            if self.chr0.is_none() {
                new_state = State::End;
                break;
            }

            let chr = self.chr0.unwrap();
            new_state = state_transition(state, chr);
            println!("chr: {chr:?} {state:?} -> {new_state:?}");

            if new_state == State::End {
                break;
            }

            if new_state == State::Error {
                let end = self.get_pos();

                value.push(chr);
                self.consume();

                return Err(LexicalError {
                    error: LexicalErrorType::IllegalLiteral { tok: chr },
                    location: SrcSpan { start, end },
                });
            }

            value.push(chr);
            self.consume();
            state = new_state;
        }

        debug_assert!(new_state == State::End);
        let end = self.get_pos();

        match state {
            State::Bin => {
                return Ok((
                    start,
                    Token::Int {
                        base: Base::Binary,
                        value,
                    },
                    end,
                ));
            }
            State::Oct => {
                return Ok((
                    start,
                    Token::Int {
                        base: Base::Octal,
                        value,
                    },
                    end,
                ));
            }
            State::Int | State::Zero => {
                return Ok((
                    start,
                    Token::Int {
                        base: Base::Decimal,
                        value,
                    },
                    end,
                ));
            }
            State::Hex => {
                return Ok((
                    start,
                    Token::Int {
                        base: Base::Hexadecimal,
                        value,
                    },
                    end,
                ));
            }
            State::Hex => {
                return Ok((
                    start,
                    Token::Int {
                        base: Base::Hexadecimal,
                        value,
                    },
                    end,
                ));
            }
            State::ExpInt => {
                return Ok((
                    start,
                    Token::Float {
                        has_exp: true,
                        value,
                    },
                    end,
                ));
            }
            State::Frac | State::Dot => {
                return Ok((
                    start,
                    Token::Float {
                        has_exp: false,
                        value,
                    },
                    end,
                ));
            }
            _ => unreachable!("Invalid state transition {state:?} -> {new_state:?}"),
        }
    }
}

#[test]
fn test_int() {
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
        let token = lexer._next().unwrap();
        assert_eq!(token, (start, expected_token, end));
    }
}

#[test]
fn test_float() {
    return;
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
            Token::Float {
                has_exp: false,
                value: "-0".into(),
            },
            35,
        ),
        (
            36,
            Token::Float {
                has_exp: false,
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
        let token = lexer._next().unwrap();
        assert_eq!(token, (start, expected_token, end));
    }
}

macro_rules! generate_float_tests {
    ($($name:ident: $input:expr => $expected:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let chars = $input.char_indices().map(|(i, c)| (i as u32, c));
                let mut lexer = Lexer::new(chars);

                let token = lexer._next().unwrap();
                assert_eq!(token, $expected);
            }
        )*
    };
}

generate_float_tests! {
    test_float_3_14: "3.14" => (
        0,
        Token::Float {
            has_exp: false,
            value: "3.14".into(),
        },
        4,
    ),
    test_float_0_5: ".5" => (
        0,
        Token::Float {
            has_exp: false,
            value: ".5".into(),
        },
        2,
    ),
    test_float_10: "10." => (
        0,
        Token::Float {
            has_exp: false,
            value: "10.".into(),
        },
        3,
    ),
    test_float_1e10: "1e10" => (
        0,
        Token::Float {
            has_exp: true,
            value: "1e10".into(),
        },
        4,
    ),
    test_float_2_9e_minus_3: "2.9e-3" => (
        0,
        Token::Float {
            has_exp: true,
            value: "2.9e-3".into(),
        },
        6,
    ),
    test_float_3E_plus_4: "3E+4" => (
        0,
        Token::Float {
            has_exp: true,
            value: "3E+4".into(),
        },
        4,
    ),
    test_float_0_0: "0.0" => (
        0,
        Token::Float {
            has_exp: false,
            value: "0.0".into(),
        },
        3,
    ),
    test_float_minus_0: "-0" => (
        0,
        Token::Int {
            base: Base::Decimal,
            value: "-0".into(),
        },
        2,
    ),
    test_float_plus_0: "+0" => (
        0,
        Token::Int{
            base: Base::Decimal,
            value: "+0".into(),
        },
        2,
    ),
    test_float_0_2: "0.2" => (
        0,
        Token::Float {
            has_exp: false,
            value: "0.2".into(),
        },
        3,
    ),
    test_float_2_123456: "2.123456" => (
        0,
        Token::Float {
            has_exp: false,
            value: "2.123456".into(),
        },
        8,
    ),
    test_float_dot_2: ".2" => (
        0,
        Token::Float {
            has_exp: false,
            value: ".2".into(),
        },
        2,
    ),
    test_float_2_dot: "2." => (
        0,
        Token::Float {
            has_exp: false,
            value: "2.".into(),
        },
        2,
    ),
    test_float_minus_2_5: "-2.5" => (
        0,
        Token::Float {
            has_exp: false,
            value: "-2.5".into(),
        },
        4,
    ),
    test_float_plus_2_5: "+2.5" => (
        0,
        Token::Float {
            has_exp: false,
            value: "+2.5".into(),
        },
        4,
    ),
    test_float_1e3: "1e3" => (
        0,
        Token::Float {
            has_exp: true,
            value: "1e3".into(),
        },
        3,
    ),
    test_float_1e_plus_3: "1e+3" => (
        0,
        Token::Float {
            has_exp: true,
            value: "1e+3".into(),
        },
        4,
    ),
    test_float_1e_minus_3: "1e-3" => (
        0,
        Token::Float {
            has_exp: true,
            value: "1e-3".into(),
        },
        4,
    ),
    test_float_minus_1e_minus_3: "-1e-3" => (
        0,
        Token::Float {
            has_exp: true,
            value: "-1e-3".into(),
        },
        5,
    ),
    test_float_plus_1e3: "+1e3" => (
        0,
        Token::Float {
            has_exp: true,
            value: "+1e3".into(),
        },
        4,
    ),
    test_float_0e0: "0e0" => (
        0,
        Token::Float {
            has_exp: true,
            value: "0e0".into(),
        },
        3,
    ),
    test_float_minus_0e0: "-0e0" => (
        0,
        Token::Float {
            has_exp: true,
            value: "-0e0".into(),
        },
        4,
    ),
    test_float_plus_0e0: "+0e0" => (
        0,
        Token::Float {
            has_exp: true,
            value: "+0e0".into(),
        },
        4,
    ),
    test_float_123_456: "123.456" => (
        0,
        Token::Float {
            has_exp: false,
            value: "123.456".into(),
        },
        7,
    ),
    test_float_1e1000: "1e1000" => (
        0,
        Token::Float {
            has_exp: true,
            value: "1e1000".into(),
        },
        6,
    ),
    test_float_1e_minus_1000: "1e-1000" => (
        0,
        Token::Float {
            has_exp: true,
            value: "1e-1000".into(),
        },
        7,
    ),
    test_float_1e_plus_1000: "1e+1000" => (
        0,
        Token::Float {
            has_exp: true,
            value: "1e+1000".into(),
        },
        7,
    ),
    test_float_1_000_000_1: "1_000.000_1" => (
        0,
        Token::Float {
            has_exp: false,
            value: "1_000.000_1".into(),
        },
        11,
    ),
    test_float_minus_1_dot: "-1." => (
        0,
        Token::Float {
            has_exp: false,
            value: "-1.".into(),
        },
        3,
    ),
    test_float_plus_1_dot: "+1." => (
        0,
        Token::Float {
            has_exp: false,
            value: "+1.".into(),
        },
        3,
    ),
}

#[test]
fn test_int2() {
    let source = "1a";
    let chars = source.char_indices().map(|(i, c)| (i as u32, c));
    let mut lexer = Lexer::new(chars);

    let expected_tokens = vec![
        (
            0,
            Token::Int {
                base: Base::Decimal,
                value: "1".into(),
            },
            1,
        ),
        (1, Token::Ident { name: "a".into() }, 2),
    ];

    for (start, expected_token, end) in expected_tokens {
        let token = lexer._next().unwrap();
        assert_eq!(token, (start, expected_token, end));
    }
}
