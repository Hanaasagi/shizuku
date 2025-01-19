use super::utils::is_decimal;
use super::utils::is_hex_decimal;
use super::utils::is_id_continue;
use super::utils::is_id_start;
use super::utils::is_whitespace;
use crate::lexer::utils::is_binary;
use crate::lexer::utils::is_octal;
use crate::token::Base;
use crate::token::Token;
use ecow::EcoString;

#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)]
pub struct SrcSpan {
    pub start: u32,
    pub end: u32,
}

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
    // IllegalLiteral { typ: LiteralType, tok: char },
    IllegalLiteral { tok: char },
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

    pub fn skip_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
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

    pub fn get_pos(&self) -> u32 {
        self.loc0
    }

    fn emit(&mut self, spanned: Spanned) {
        self.pending.push(spanned);
    }

    pub(super) fn _next(&mut self) -> LexResult {
        while self.pending.is_empty() {
            self.advance_token()?;
        }

        Ok(self.pending.remove(0))
    }
}

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

impl<I> Lexer<I>
where
    I: Iterator<Item = (LOC, char)>,
{
    fn advance_token(&mut self) -> Result<(), LexicalError> {
        self.skip_while(is_whitespace);
        if let Some(c) = self.chr0 {
            self._advance_token()?;
        } else {
            let tok_pos = self.get_pos();
            self.emit((tok_pos, Token::EOF, tok_pos));
        }
        Ok(())
    }

    fn consume_single_char_token(&mut self, expected_token: Token) {
        let start = self.get_pos();
        let _ = self.consume().expect("Failed to consume char");
        let end = self.get_pos();
        self.emit((start, expected_token, end));
    }

    fn _advance_token(&mut self) -> Result<(), LexicalError> {
        debug_assert!(self.chr0.is_some());

        let chr = self.chr0.unwrap();

        match chr {
            // Single Char Token
            '(' => {
                self.consume_single_char_token(Token::LParen);
            }
            ')' => {
                self.consume_single_char_token(Token::RParen);
            }
            '[' => {
                self.consume_single_char_token(Token::LBracket);
            }
            ']' => {
                self.consume_single_char_token(Token::RBracket);
            }
            '{' => {
                self.consume_single_char_token(Token::LBrace);
            }
            '}' => {
                self.consume_single_char_token(Token::RBrace);
            }
            ':' => {
                self.consume_single_char_token(Token::Colon);
            }
            '@' => {
                self.consume_single_char_token(Token::At);
            }
            '%' => {
                self.consume_single_char_token(Token::Percent);
            }
            ',' => {
                self.consume_single_char_token(Token::Comma);
            }
            '.' if self.chr1.is_some() && !is_decimal(self.chr1.unwrap()) => {
                self.consume_single_char_token(Token::Dot);
            }
            '#' => {
                self.consume_single_char_token(Token::Hash);
            }
            ';' => {
                self.consume_single_char_token(Token::Semicolon);
            }
            '+' if self.chr1.is_some()
                && !(self.chr1.unwrap().is_digit(10) || self.chr1.unwrap() == '.') =>
            {
                self.consume_single_char_token(Token::Plus);
            }
            // '\n' | ' ' | '\t' | '\x0C' => {
            //     let start = self.get_pos();
            //     let _ = self.consume();
            //     let end = self.get_pos();
            //     if chr == '\n' {
            //         self.emit((start, Token::NewLine, end));
            //     }
            // }

            // Multi Char Token
            //
            '-' if self.chr1.is_some()
                && !(self.chr1.unwrap().is_digit(10) || self.chr1.unwrap() == '.') =>
            {
                let start = self.get_pos();
                let _ = self.consume();
                match self.chr0 {
                    Some('>') => {
                        let _ = self.consume();
                        let end = self.get_pos();
                        self.emit((start, Token::RArrow, end));
                    }
                    _ => {
                        let end = self.get_pos();
                        self.emit((start, Token::Minus, end));
                    }
                }
            }
            '=' => {
                let start = self.get_pos();
                let _ = self.consume();
                match self.chr0 {
                    Some('=') => {
                        let _ = self.consume();
                        let end = self.get_pos();
                        self.emit((start, Token::EqualEqual, end));
                    }
                    _ => {
                        let end = self.get_pos();
                        self.emit((start, Token::Equal, end));
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
                        unimplemented!("");
                    }
                }
                let _ = self.consume();
            }
            c if is_id_start(c) => {
                let id_or_keyword = self.consume_ident_or_keyword();
                self.emit(id_or_keyword);
            }
            //
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
}

#[test]
fn test_comment() {
    fn test_lexer_comment(source: &str, expected_token: (u32, Token, u32)) {
        let chars = source.char_indices().map(|(i, c)| (i as u32, c));
        let mut lexer = Lexer::new(chars);

        let token = lexer._next().unwrap();

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

        let token = lexer._next().unwrap();

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

impl<I> Lexer<I>
where
    I: Iterator<Item = (LOC, char)>,
{
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
}

#[test]
fn test_ident() {
    let source = " vAri4ble_ ";
    let chars = source.char_indices().map(|(i, c)| (i as u32, c));
    let mut lexer = Lexer::new(chars);
    let token = lexer._next().unwrap();

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
    let token = lexer._next().unwrap();

    assert_eq!(token, (1, Token::Fn, (1 + "fn".len()) as u32));
}

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
        (34, Token::RArrow, 36),
        (37, Token::Ident { name: "i32".into() }, 40),
        (41, Token::LBrace, 42),
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
        (82, Token::Return, 88),
        (89, Token::Ident { name: "sum".into() }, 92),
        (92, Token::Semicolon, 93),
        (98, Token::RBrace, 99),
        (104, Token::EOF, 104),
    ];

    for (start, token, end) in expected_tokens {
        let actual = lexer._next().unwrap();
        assert_eq!(actual, (start, token, end));
    }
}
