use super::lexer::LexicalErrorType::*;
use super::{lexer::*, utils::is_whitespace};
use crate::token::Base;
use crate::token::Token;
use ecow::EcoString;

/// ```text
/// START:
///     "+" | "-" -> SIGN
///     "0" -> ZERO
///     "1".."9" -> INT
///     "." -> DOT  // e.g. `.2`
///     .. -> ERROR
///
/// SIGN:
///     "1".."9" -> INT
///     "0" -> ZERO
///     "." -> DOT  // e.g. `-.2`
///     .. -> ERROR
///
/// ZERO:
///     "x" | "X" -> HEX
///     "o" | "O" -> OCT
///     "b" | "B" -> BIN
///     "." -> DOT
///     "e" | "E" -> EXP  // e.g. `0e1`
///     WHITESPACE | EOF -> END
///     "0"-> ZERO  // e.g. `02` is invalid but `00` is valid
///     .. -> ERROR
///
/// INT:
///     "0".."9"-> INT
///     "." -> DOT
///     "e" | "E" -> EXP
///     WHITESPACE | EOF -> END
///     "_"  -> INT_UNDERSCORE
///     .. -> ERROR
///
/// DOT:
///     "0".."9" -> FRAC
///     "e" | "E" -> EXP  // e.g. `.2e1`
///     WHITESPACE | EOF -> END
///     .. -> ERROR
///
/// FRAC:
///     "0".."9" -> FRAC
///     "e" | "E" -> EXP
///     "_"  -> FRAC_UNDERSCORE
///     WHITESPACE | EOF -> END
///     .. -> ERROR
///
/// EXP:
///     "+" | "-" -> EXP_SIGN
///     "0".."9" -> EXP_INT
///     .. -> ERROR
///
/// EXP_SIGN:
///     "0".."9"-> EXP_INT
///     .. -> ERROR
///
/// EXP_INT:
///     "0".."9" -> EXP_INT
///     "_"  -> EXP_INT_UNDERSCORE
///     WHITESPACE | EOF -> END
///     .. -> ERROR
///
/// HEX:
///     "0".."9" | "a".."f" | "A".."F" -> HEX
///     "_"  -> HEX_UNDERSCORE
///     WHITESPACE | EOF -> END
///     .. -> ERROR
///
/// OCT:
///     "0".."7" -> OCT
///     "_"  -> OCT_UNDERSCORE
///     WHITESPACE | EOF -> END
///     .. -> ERROR
///
/// BIN:
///     "0" | "1" -> BIN
///     "_"  -> BIN_UNDERSCORE
///     WHITESPACE | EOF -> END
///     .. -> ERROR
///
/// INT_UNDERSCORE:
///     "0".."9" -> INT
///     .. -> ERROR
///
/// EXP_INT_UNDERSCORE:
///     "0".."9" -> EXP_INT
///     .. -> ERROR
///
/// FRAC_UNDERSCORE:
///     "0".."9" -> FRAC
///     .. -> ERROR
///
/// HEX_UNDERSCORE:
///     "0".."9" | "a".."f" | "A".."F" -> HEX
///     .. -> ERROR
///
/// OCT_UNDERSCORE:
///     "0".."7" -> OCT
///     .. -> ERROR
///
/// BIN_UNDERSCORE:
///     "0" | "1" -> BIN
///     .. -> ERROR
///
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Start,
    Sign,
    Zero,
    Int,
    Dot,
    Frac,
    Exp,
    ExpSign,
    ExpInt,
    Hex,
    Oct,
    Bin,

    IntUnderscore,
    ExpIntUnderscore,
    FracUnderscore,
    HexUnderscore,
    OctUnderscore,
    BinUnderscore,

    // FinalState
    End,
    // FinalState
    Error,
}

fn state_transition(state: State, chr: Option<char>) -> State {
    // handle EOF
    if chr.is_none() || is_whitespace(chr.unwrap()) {
        if matches!(
            state,
            State::Zero
                | State::Int
                | State::Dot
                | State::Frac
                | State::ExpInt
                | State::Hex
                | State::Oct
                | State::Bin
        ) {
            return State::End;
        } else {
            return State::Error;
        }
    }

    let chr = chr.unwrap();

    match state {
        State::Start => {
            if chr == '+' || chr == '-' {
                State::Sign
            } else if chr == '0' {
                State::Zero
            } else if chr.is_ascii_digit() {
                State::Int
            } else if chr == '.' {
                State::Dot
            } else {
                State::Error
            }
        }
        State::Sign => {
            if chr == '0' {
                State::Zero
            } else if chr.is_ascii_digit() {
                State::Int
            } else if chr == '.' {
                State::Dot
            } else {
                State::Error
            }
        }
        State::Zero => match chr {
            'x' | 'X' => State::Hex,
            'o' | 'O' => State::Oct,
            'b' | 'B' => State::Bin,
            '.' => State::Dot,
            'e' | 'E' => State::Exp,
            '0' => State::Zero,
            _ => State::Error,
        },
        State::Int => {
            if chr.is_ascii_digit() {
                State::Int
            } else if chr == '.' {
                State::Dot
            } else if chr == 'e' || chr == 'E' {
                State::Exp
            } else if chr == '_' {
                State::IntUnderscore
            } else {
                State::Error
            }
        }
        State::Dot => {
            if chr.is_ascii_digit() {
                State::Frac
            } else {
                State::Error
            }
        }
        State::Frac => {
            if chr.is_ascii_digit() {
                State::Frac
            } else if chr == 'e' || chr == 'E' {
                State::Exp
            } else if chr == '_' {
                State::FracUnderscore
            } else {
                State::Error
            }
        }
        State::Exp => {
            if chr == '+' || chr == '-' {
                State::ExpSign
            } else if chr.is_ascii_digit() {
                State::ExpInt
            } else {
                State::Error
            }
        }
        State::ExpSign => {
            if chr.is_ascii_digit() {
                State::ExpInt
            } else {
                State::Error
            }
        }
        State::ExpInt => {
            if chr.is_ascii_digit() {
                State::ExpInt
            } else if chr == '_' {
                State::ExpIntUnderscore
            } else {
                State::Error
            }
        }
        State::Hex => {
            if chr.is_ascii_hexdigit() {
                State::Hex
            } else if chr == '_' {
                State::HexUnderscore
            } else {
                State::Error
            }
        }
        State::Oct => {
            if chr.is_ascii_octdigit() {
                State::Oct
            } else if chr == '_' {
                State::OctUnderscore
            } else {
                State::Error
            }
        }
        State::Bin => {
            if chr == '0' || chr == '1' {
                State::Bin
            } else if chr == '_' {
                State::BinUnderscore
            } else {
                State::Error
            }
        }
        State::IntUnderscore => {
            if chr.is_ascii_digit() {
                State::Int
            } else {
                State::Error
            }
        }
        State::FracUnderscore => {
            if chr.is_ascii_digit() {
                State::Frac
            } else {
                State::Error
            }
        }
        State::ExpIntUnderscore => {
            if chr.is_ascii_digit() {
                State::ExpInt
            } else {
                State::Error
            }
        }
        State::HexUnderscore => {
            if chr.is_ascii_hexdigit() {
                State::Hex
            } else {
                State::Error
            }
        }
        State::OctUnderscore => {
            if chr >= '0' && chr <= '7' {
                State::Oct
            } else {
                State::Error
            }
        }
        State::BinUnderscore => {
            if chr == '0' || chr == '1' {
                State::Bin
            } else {
                State::Error
            }
        }
        State::End | State::Error => {
            panic!("{state:?} is the final state.");
        }
    }
}

impl<I> Lexer<I>
where
    I: Iterator<Item = (LOC, char)>,
{
    pub(super) fn consume_number_like(&mut self) -> LexResult {
        // At least one char
        debug_assert!(self.chr0.is_some());

        let mut state = State::Start;
        let mut value = EcoString::new();
        let start = self.get_pos();

        let mut new_state;

        let mut prev_chr = None;
        loop {
            let chr = self.chr0;
            new_state = state_transition(state, chr);
            println!("chr: {chr:?} {state:?} -> {new_state:?}");

            debug_assert!(
                chr.is_some()
                    || (chr.is_none() && (new_state == State::End || new_state == State::Error))
            );

            if new_state == State::End {
                break;
            }

            if new_state == State::Error {
                if chr.is_none() {
                    let end = self.get_pos();

                    return Err(LexicalError {
                        error: IllegalLiteral {
                            tok: prev_chr.unwrap(),
                        },
                        location: SrcSpan { start, end },
                    });
                }

                value.push(chr.unwrap());
                self.consume();
                let end = self.get_pos();

                return Err(LexicalError {
                    error: IllegalLiteral { tok: chr.unwrap() },
                    location: SrcSpan { start, end },
                });
            }

            // safe unwrap
            value.push(chr.expect("None should be handled in state transition"));
            self.consume();
            state = new_state;
            prev_chr = chr;
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

#[cfg(test)]
mod number_tests {
    use super::*;

    macro_rules! generate_valid_number_tests {
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

    macro_rules! generate_invalid_number_tests{
        ($($name:ident: $input:expr => $expected:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let chars = $input.char_indices().map(|(i, c)| (i as u32, c));
                    let mut lexer = Lexer::new(chars);

                    let token = lexer._next().unwrap_err();
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
            let token = lexer._next().unwrap();
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
            let token = lexer._next().unwrap();
            assert_eq!(token, (start, expected_token, end));
        }
    }
}
