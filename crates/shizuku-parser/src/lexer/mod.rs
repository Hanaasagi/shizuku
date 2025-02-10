mod number;
mod utils;

use crate::span::SrcSpan;
use crate::token::Base;
use crate::token::Token;
use ecow::EcoString;
use number::State;
use number::state_transition;
use utils::is_id_continue;
use utils::is_id_start;
use utils::is_whitespace;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LiteralType {
    String,
    Binary,
    Octal,
    Decimal,
    Hexadecimal,
    Float,
    ExpFloat,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LexicalErrorType {
    UnexpectedStringEnd, // Unterminated string literal
    UnrecognizedToken { tok: char },
    IllegalLiteral { tok: char },
    UnexpectedCharEnd, // Unterminated char literal
    EmptyCharLiteral,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LexicalError {
    pub error: LexicalErrorType,
    pub location: SrcSpan,
}

pub type LOC = u32;
pub type Spanned = (LOC, Token, LOC);
pub type LexResult = Result<Spanned, LexicalError>;

/// A lexer for the Shizuku language.
pub struct Lexer<I>
where
    I: Iterator<Item = (LOC, char)>,
{
    stream: I,

    pub pending: Vec<Spanned>,

    pub chr0: Option<char>,
    pub chr1: Option<char>,
    pub loc0: LOC,
    pub loc1: LOC,
    pub location: LOC,
}

impl<I> Lexer<I>
where
    I: Iterator<Item = (LOC, char)>,
{
    /// Creates a new lexer from the given stream
    pub fn new(stream: I) -> Self {
        let mut lexer = Self {
            stream,
            pending: Vec::new(),
            location: 0,
            // current char
            chr0: None,
            loc0: 0,
            // next char
            chr1: None,
            loc1: 0,
        };
        let _ = lexer.consume();
        let _ = lexer.consume();
        lexer.location = 0;
        lexer
    }

    fn skip_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while self.chr0.is_some_and(&mut predicate) {
            self.consume();
        }
    }

    pub fn consume(&mut self) -> Option<char> {
        let chr = self.chr0;
        let next_char = match self.stream.next() {
            Some((loc, c)) => {
                self.loc0 = self.loc1;
                self.loc1 = loc;
                Some(c)
            }
            None => {
                // EOF needs a single advance
                self.loc0 = self.loc1;
                self.loc1 += 1;
                None
            }
        };
        self.chr0 = self.chr1;
        self.chr1 = next_char;
        chr
    }

    /// Returns true if the next char is not None and matches the given predicate
    fn next_chr_is(&self, mut predicate: impl FnMut(char) -> bool) -> bool {
        self.chr1.is_some_and(&mut predicate)
    }

    pub fn get_pos(&self) -> u32 {
        self.loc0
    }

    fn emit(&mut self, spanned: Spanned) {
        self.pending.push(spanned);
    }

    pub fn next(&mut self) -> LexResult {
        while self.pending.is_empty() {
            self.advance_token()?;
        }

        Ok(self.pending.remove(0))
    }

    /// Consumes `char_count` characters and emits `expected_token`
    fn consume_expect_token(&mut self, expected_token: Token, char_count: u32) {
        let start_pos = self.get_pos();
        for _ in 0..char_count {
            let _ = self.consume().expect("Failed to consume char");
        }
        let end_pos = self.get_pos();
        self.emit((start_pos, expected_token, end_pos));
    }

    fn _advance_token(&mut self) -> Result<(), LexicalError> {
        debug_assert!(self.chr0.is_some());

        let chr = self.chr0.unwrap();

        match chr {
            // Single Char Token
            '(' => {
                self.consume_expect_token(Token::LParen, 1);
            }
            ')' => {
                self.consume_expect_token(Token::RParen, 1);
            }
            '[' => {
                self.consume_expect_token(Token::LBracket, 1);
            }
            ']' => {
                self.consume_expect_token(Token::RBracket, 1);
            }
            '{' => {
                self.consume_expect_token(Token::LBrace, 1);
            }
            '}' => {
                self.consume_expect_token(Token::RBrace, 1);
            }
            ':' => {
                self.consume_expect_token(Token::Colon, 1);
            }
            '@' => {
                self.consume_expect_token(Token::At, 1);
            }
            '%' => {
                self.consume_expect_token(Token::Percent, 1);
            }
            ',' => {
                self.consume_expect_token(Token::Comma, 1);
            }
            '#' => {
                self.consume_expect_token(Token::Hash, 1);
            }
            ';' => {
                self.consume_expect_token(Token::Semicolon, 1);
            }
            '&' => {
                self.consume_expect_token(Token::Amper, 1);
            }
            '?' => {
                self.consume_expect_token(Token::Question, 1);
            }
            // Multi Char Token
            //
            // `+1` / `+.2` is number Token
            '+' if !(self.next_chr_is(|c| c.is_ascii_digit() || c == '.')) => {
                self.consume_expect_token(Token::Plus, 1);
            }
            // `-1` / `-.2` is number Token
            '-' if !(self.next_chr_is(|c| c.is_ascii_digit() || c == '.')) => {
                // handle `->`
                match self.chr1 {
                    Some('>') => {
                        self.consume_expect_token(Token::MinusRArrow, 2);
                    }
                    _ => {
                        self.consume_expect_token(Token::Minus, 1);
                    }
                }
            }
            '=' => {
                // handl `==`
                match self.chr1 {
                    Some('=') => {
                        self.consume_expect_token(Token::Equal2, 2);
                    }
                    _ => {
                        self.consume_expect_token(Token::Equal, 1);
                    }
                }
            }
            '!' => {
                // handle `!=` or `!`
                match self.chr1 {
                    Some('=') => {
                        self.consume_expect_token(Token::ExclamationEqual, 2);
                    }
                    _ => {
                        self.consume_expect_token(Token::Exclamation, 1);
                    }
                }
            }
            '|' => {
                // handle `|` or `|>`
                match self.chr1 {
                    Some('>') => {
                        self.consume_expect_token(Token::PipeRArrow, 2);
                    }
                    _ => {
                        self.consume_expect_token(Token::Pipe, 1);
                    }
                }
            }
            '<' => {
                // handle `<` or `<=` or `<-`
                match self.chr1 {
                    Some('=') => {
                        self.consume_expect_token(Token::LArrowEqual, 2);
                    }
                    Some('-') => {
                        self.consume_expect_token(Token::LArrowMinus, 2);
                    }
                    _ => {
                        self.consume_expect_token(Token::LArrow, 1);
                    }
                }
            }
            '>' => {
                // handle `>` or `>=`
                match self.chr1 {
                    Some('=') => {
                        self.consume_expect_token(Token::RArrowEqual, 2);
                    }
                    _ => {
                        self.consume_expect_token(Token::RArrow, 1);
                    }
                }
            }
            '.' if !(self.next_chr_is(|c| c.is_ascii_digit())) => {
                // handle `..` and `.`
                match self.chr1 {
                    Some('.') => {
                        self.consume_expect_token(Token::Dot2, 2);
                    }
                    _ => {
                        self.consume_expect_token(Token::Dot, 1);
                    }
                }
            }
            '/' => {
                // handle //
                match self.chr1 {
                    Some('/') => {
                        let comment = self.consume_comment_or_doc();
                        self.emit(comment);
                    }
                    _ => {
                        self.consume_expect_token(Token::Slash, 1);
                    }
                }
            }
            '"' => {
                let string_lit = self.consume_string_literal()?;
                self.emit(string_lit);
            }
            '\'' => {
                let char_lit = self.consume_char_literal()?;
                self.emit(char_lit);
            }
            c if is_id_start(c) => {
                let id_or_keyword = self.consume_ident_or_keyword();
                self.emit(id_or_keyword);
            }
            // handle integer or float
            '0'..='9' | '.' | '-' | '+' => {
                let number_like = self.consume_number_like()?;
                self.emit(number_like);
            }
            c => {
                let location = self.get_pos();
                return Err(LexicalError {
                    error: LexicalErrorType::UnrecognizedToken { tok: c },
                    location: SrcSpan {
                        start: location,
                        end: location,
                    },
                });
            }
        }

        Ok(())
    }

    fn advance_token(&mut self) -> Result<(), LexicalError> {
        while let Some(c) = self.chr0 {
            if is_whitespace(c) {
                if c == '\n' {
                    let start = self.get_pos();
                    self.consume();
                    let end = self.get_pos();
                    self.emit((start, Token::NewLine, end));
                } else {
                    self.consume();
                }
            } else {
                break;
            }
        }
        if let Some(c) = self.chr0 {
            self._advance_token()?;
        } else {
            let tok_pos = self.get_pos();
            self.emit((tok_pos, Token::EOF, tok_pos));
        }
        Ok(())
    }
}

impl<I> Lexer<I>
where
    I: Iterator<Item = (LOC, char)>,
{
    fn consume_comment_or_doc(&mut self) -> Spanned {
        enum Kind {
            Comment,
            Doc,
        }

        debug_assert!(self.chr0 == Some('/'));
        debug_assert!(self.chr1 == Some('/'));

        self.consume();

        let kind = match self.chr1 {
            Some('/') => {
                let _ = self.consume();
                let _ = self.consume();
                Kind::Doc
            }
            _ => {
                let _ = self.consume();
                Kind::Comment
            }
        };

        let mut content = EcoString::new();

        let start_pos = self.get_pos();
        while self.chr0 != Some('\n') {
            match self.chr0 {
                Some(c) => content.push(c),
                None => break,
            }
            let _ = self.consume();
        }

        let end_pos = self.get_pos();

        let token = match kind {
            Kind::Comment => Token::Comment { content },
            Kind::Doc => Token::CommentDoc { content },
        };

        (start_pos, token, end_pos)
    }
    fn is_name_continuation(&self) -> bool {
        self.chr0
            .map(|c| matches!(c, '_' | '0'..='9' | 'a'..='z' | 'A'..='Z'))
            .unwrap_or(false)
    }

    fn consume_ident_or_keyword(&mut self) -> Spanned {
        debug_assert!(self.chr0.is_some());
        debug_assert!(is_id_start(self.chr0.unwrap()));

        let mut name = EcoString::new();

        let start = self.get_pos();
        name.push(self.chr0.unwrap());
        self.consume();
        while let Some(chr) = self.chr0 {
            if is_id_continue(chr) {
                name.push(chr);
                self.consume();
            } else {
                break;
            }
        }
        let end = self.get_pos();

        if let Some(token) = Token::try_from_keywords(&name) {
            (start, token, end)
        } else {
            (start, Token::Ident { name }, end)
        }
    }
    fn consume_char_literal(&mut self) -> Result<Spanned, LexicalError> {
        debug_assert!(self.chr0 == Some('\''));

        let start = self.get_pos();
        self.consume();

        let chr = match self.chr0 {
            Some('\'') => {
                self.consume();
                return Err(LexicalError {
                    error: LexicalErrorType::EmptyCharLiteral,
                    location: SrcSpan {
                        start,
                        end: self.get_pos(),
                    },
                });
            }
            Some(c) => {
                self.consume();
                c
            }
            None => {
                return Err(LexicalError {
                    error: LexicalErrorType::UnexpectedCharEnd,
                    location: SrcSpan {
                        start,
                        end: start + 1,
                    },
                });
            }
        };

        if self.chr0 != Some('\'') {
            return Err(LexicalError {
                error: LexicalErrorType::UnexpectedCharEnd,
                location: SrcSpan {
                    start,
                    end: self.get_pos(),
                },
            });
        }

        self.consume(); // Consume closing quote
        let end = self.get_pos();

        Ok((start, Token::Char { value: chr }, end))
    }

    fn consume_string_literal(&mut self) -> Result<Spanned, LexicalError> {
        debug_assert!(self.chr0 == Some('"'));

        let start = self.get_pos();
        self.consume(); // Consume opening quote

        let mut value = EcoString::new();

        while let Some(c) = self.chr0 {
            if c == '"' {
                break;
            }
            value.push(c);
            self.consume();
        }

        if self.chr0 != Some('"') {
            return Err(LexicalError {
                error: LexicalErrorType::UnexpectedStringEnd,
                location: SrcSpan {
                    start,
                    end: self.get_pos(),
                },
            });
        }

        self.consume(); // Consume closing quote
        let end = self.get_pos();

        Ok((start, Token::String { value }, end))
    }

    fn consume_number_like(&mut self) -> LexResult {
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
                        error: LexicalErrorType::IllegalLiteral {
                            tok: prev_chr.unwrap(),
                        },
                        location: SrcSpan { start, end },
                    });
                }

                value.push(chr.unwrap());
                self.consume();
                let end = self.get_pos();

                return Err(LexicalError {
                    error: LexicalErrorType::IllegalLiteral { tok: chr.unwrap() },
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
mod core_function_tests {
    use super::*;
    #[test]
    fn test_chr0_chr1() {
        let source = "string";
        let chars = source.char_indices().map(|(i, c)| (i as u32, c));
        let lexer = Lexer::new(chars);

        assert_eq!(lexer.get_pos(), 0);
        assert_eq!(lexer.chr0, Some('s'));
        assert_eq!(lexer.loc0, 0);
        assert_eq!(lexer.loc1, 1);
        assert_eq!(lexer.chr1, Some('t'));
        assert_eq!(lexer.get_pos(), 0);
    }

    #[test]
    fn test_consume() {
        let source = "string";
        let chars = source.char_indices().map(|(i, c)| (i as u32, c));
        let mut lexer = Lexer::new(chars);

        assert_eq!(lexer.get_pos(), 0);
        assert_eq!(lexer.consume(), Some('s'));
        assert_eq!(lexer.get_pos(), 1);

        assert_eq!(lexer.consume(), Some('t'));
        assert_eq!(lexer.get_pos(), 2);
    }

    #[test]
    fn test_skip_while() {
        let source = "    string";
        let chars = source.char_indices().map(|(i, c)| (i as u32, c));
        let mut lexer = Lexer::new(chars);

        lexer.skip_while(is_whitespace);

        assert_eq!(
            lexer.get_pos(),
            source.chars().position(|c| c == 's').unwrap() as u32
        );
        assert_eq!(lexer.chr0, Some('s'));
        assert_eq!(lexer.chr1, Some('t'));
    }
}

#[cfg(test)]
mod token_tests {
    use super::*;

    macro_rules! test_single_token {
        ($name:ident, $source:expr, $expected_token:expr) => {
            #[test]
            fn $name() {
                let chars = $source.char_indices().map(|(i, c)| (i as u32, c));
                let mut lexer = Lexer::new(chars);

                let token = lexer.next().unwrap();

                assert_eq!(token.0, 0);
                assert_eq!(token.1, $expected_token);
                assert_eq!(token.2, $source.len() as u32);
            }
        };
    }

    macro_rules! test_keyword {
        ($name:ident, $source:expr, $expected_token:expr) => {
            test_single_token!($name, $source, $expected_token);
        };
    }

    test_single_token!(test_lparen, "(", Token::LParen);
    test_single_token!(test_rparen, ")", Token::RParen);
    test_single_token!(test_lbracket, "[", Token::LBracket);
    test_single_token!(test_rbracket, "]", Token::RBracket);
    test_single_token!(test_lbrace, "{", Token::LBrace);
    test_single_token!(test_rbrace, "}", Token::RBrace);
    test_single_token!(test_colon, ":", Token::Colon);
    test_single_token!(test_at, "@", Token::At);
    test_single_token!(test_percent, "%", Token::Percent);
    test_single_token!(test_comma, ",", Token::Comma);
    test_single_token!(test_hash, "#", Token::Hash);
    test_single_token!(test_semicolon, ";", Token::Semicolon);
    test_single_token!(test_amper, "&", Token::Amper);
    test_single_token!(test_question, "?", Token::Question);

    test_single_token!(test_plus, "+", Token::Plus);
    test_single_token!(test_minus, "-", Token::Minus);
    test_single_token!(test_rarrow, "->", Token::MinusRArrow);
    test_single_token!(test_equal, "=", Token::Equal);
    test_single_token!(test_equal_equal, "==", Token::Equal2);
    test_single_token!(test_band, "!", Token::Exclamation);
    test_single_token!(test_not_equal, "!=", Token::ExclamationEqual);
    test_single_token!(test_vbar, "|", Token::Pipe);
    test_single_token!(test_pipe, "|>", Token::PipeRArrow);
    test_single_token!(test_lessthan, "<", Token::LArrow);
    test_single_token!(test_lessthan_equal, "<=", Token::LArrowEqual);
    test_single_token!(test_larrow, "<-", Token::LArrowMinus);
    test_single_token!(test_greathan, ">", Token::RArrow);
    test_single_token!(test_greathan_equal, ">=", Token::RArrowEqual);
    test_single_token!(test_dot, ".", Token::Dot);
    test_single_token!(test_dotdot, "..", Token::Dot2);
    test_single_token!(test_slash, "/", Token::Slash);

    #[test]
    fn test_ident() {
        let source = " vAri4ble_ ";
        let chars = source.char_indices().map(|(i, c)| (i as u32, c));
        let mut lexer = Lexer::new(chars);
        let token = lexer.next().unwrap();

        assert_eq!(
            token,
            (
                1,
                Token::Ident {
                    name: "vAri4ble_".into()
                },
                (1 + "vAri4ble_".len()) as u32
            )
        );
    }
    #[test]
    fn test_keyword() {
        let source = " fn func()";
        let chars = source.char_indices().map(|(i, c)| (i as u32, c));
        let mut lexer = Lexer::new(chars);
        let token = lexer.next().unwrap();

        assert_eq!(token, (1, Token::Fn, (1 + "fn".len()) as u32));
    }

    test_keyword!(test_as, "as", Token::As);
    test_keyword!(test_const, "const", Token::Const);
    test_keyword!(test_fn, "fn", Token::Fn);
    test_keyword!(test_if, "if", Token::If);
    test_keyword!(test_else, "else", Token::Else);
    test_keyword!(test_and, "and", Token::And);
    test_keyword!(test_or, "or", Token::Or);
    test_keyword!(test_import, "import", Token::Import);
    test_keyword!(test_let, "let", Token::Let);
    test_keyword!(test_type, "type", Token::Type);
    test_keyword!(test_opaque, "opaque", Token::Opaque);
    test_keyword!(test_pub, "pub", Token::Pub);
    test_keyword!(test_struct, "struct", Token::Struct);
    test_keyword!(test_enum, "enum", Token::Enum);
    test_keyword!(test_break, "break", Token::Break);
    test_keyword!(test_continue, "continue", Token::Continue);
    test_keyword!(test_async, "async", Token::Async);
    test_keyword!(test_await, "await", Token::Await);
    test_keyword!(test_retrun, "return", Token::Return);
    test_keyword!(test_test, "test", Token::Test);

    macro_rules! test_string_literal {
        ($name:ident, $source:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let chars = $source.char_indices().map(|(i, c)| (i as u32, c));
                let mut lexer = Lexer::new(chars);

                let token = lexer.next().unwrap();
                assert_eq!(token, $expected);
            }
        };
    }

    macro_rules! test_invalid_string_literal {
        ($name:ident, $source:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let chars = $source.char_indices().map(|(i, c)| (i as u32, c));
                let mut lexer = Lexer::new(chars);

                let token = lexer.next().unwrap_err();
                assert_eq!(token, $expected);
            }
        };
    }

    test_string_literal!(
        test_string_literal,
        r#""hello world""#,
        (
            0,
            Token::String {
                value: "hello world".into()
            },
            r#""hello world""#.len() as u32
        )
    );

    test_string_literal!(
        test_empty_string_literal,
        r#""""#,
        (0, Token::String { value: "".into() }, r#""""#.len() as u32)
    );

    test_string_literal!(
        test_char_literal,
        "'a'",
        (0, Token::Char { value: 'a' }, "'a'".len() as u32)
    );

    test_string_literal!(
        test_special_char_literal,
        "'\n'",
        (0, Token::Char { value: '\n' }, "'\n'".len() as u32)
    );

    test_invalid_string_literal!(
        test_unterminated_string_literal,
        r#""hello world"#,
        LexicalError {
            error: LexicalErrorType::UnexpectedStringEnd,
            location: SrcSpan {
                start: 0,
                end: r#""hello world"#.len() as u32
            }
        }
    );

    test_invalid_string_literal!(test_unterminated_char_literal, "'a", LexicalError {
        error: LexicalErrorType::UnexpectedCharEnd,
        location: SrcSpan { start: 0, end: 2 }
    });

    test_invalid_string_literal!(test_empty_char_literal, "''", LexicalError {
        error: LexicalErrorType::EmptyCharLiteral,
        location: SrcSpan { start: 0, end: 2 }
    });
}
