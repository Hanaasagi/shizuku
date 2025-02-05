use super::utils::is_whitespace;
use crate::token::Base;
use crate::token::Token;

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
pub(super) enum State {
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

pub(super) fn state_transition(state: State, chr: Option<char>) -> State {
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
