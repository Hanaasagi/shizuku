use ecow::EcoString;

/// Base of numeric literal encoding according to its prefix.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Base {
    /// Literal starts with "0b".
    Binary = 2,
    /// Literal starts with "0o".
    Octal = 8,
    /// Literal doesn't contain a prefix.
    Decimal = 10,
    /// Literal starts with "0x".
    Hexadecimal = 16,
}

/// Represents the various kinds of tokens that can appear in the source code.
/// Tokens are the basic building blocks of the language, including literals,
/// identifiers, operators, delimiters, and keywords.
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Identifiers and literals
    /// Identifier (e.g., variable names, function names)
    Ident {
        name: EcoString,
    },
    /// Integer literal (e.g., `123`)
    Int {
        base: Base,
        value: EcoString,
    },
    /// Floating-point literal (e.g., `3.14`)
    Float {
        has_exp: bool,
        value: EcoString,
    },
    /// String literal (e.g., `"hello"`)
    String {
        value: EcoString,
    },
    /// Single-line comment (e.g., `// comment`)
    Comment {
        content: EcoString,
    },
    /// Documentation comment (e.g., `/// doc comment`)
    CommentDoc {
        content: EcoString,
    },

    // Delimiters
    /// Left parenthesis `(`
    LParen,
    /// Right parenthesis `)`
    RParen,
    /// Left bracket `[`
    LBracket,
    /// Right bracket `]`
    RBracket,
    /// Left brace `{`
    LBrace,
    /// Right brace `}`
    RBrace,
    /// Semicolon `;`
    Semicolon,

    // Operators
    /// Addition operator `+`
    Plus,
    /// Subtraction operator `-`
    Minus,
    /// Multiplication operator `*`
    Star,
    /// Division operator `/`
    Slash,
    /// Less than operator `<`
    LessThan,
    /// Greater than operator `>`
    GreaterThan,
    /// Less than or equal operator `<=`
    LessThanEqual,
    /// Greater than or equal operator `>=`
    GreaterThanEqual,
    /// Modulus operator `%`
    Percent,

    // Miscellaneous symbols
    /// Colon `:`
    Colon,
    /// Comma `,`
    Comma,
    /// Hash symbol `#`
    Hash,
    /// Exclamation mark `!`
    Bang,
    /// Equals sign `=`
    Equal,
    /// Equality comparison `==`
    EqualEqual,
    /// Inequality comparison `!=`
    NotEqual,
    /// Bitwise OR `|`
    Vbar,
    /// Bitwise AND `&`
    Amper,
    /// Left shift `<<`
    LShift,
    /// Right shift `>>`
    RShit,
    /// Pipe operator `|>`
    Pipe,
    /// Dot `.`
    Dot,
    /// Left arrow `<-`
    LArrow,
    /// Right arrow `->`
    RArrow,
    /// Range operator `..`
    DotDot,
    /// At symbol `@`
    At,
    /// End of file token
    EOF,
    /// Question mark `?`
    Question,
    /// Exclamation mark `!`
    Exclamation,

    // Control characters
    /// Newline character
    NewLine,

    // Keywords
    // `as` keyword
    As,
    /// `const` keyword
    Const,
    /// `fn` keyword
    Fn,
    /// `if` keyword
    If,
    /// `else` keyword
    Else,
    /// `else if` keyword
    ElseIf,
    /// `and` keyword
    And,
    /// `or` keyword
    Or,
    /// `import` keyword
    Import,
    /// `let` keyword
    Let,
    /// `type` keyword
    Type,
    /// `opaque` keyword
    Opaque,
    /// `pub` keyword
    Pub,
    /// `struct` keyword
    Struct,
    /// `enum` keyword
    Enum,
    /// `break` keyword
    Break,
    /// `continue` keyword
    Continue,
    /// `async` keyword
    Async,
    /// `await` keyword
    Await,
    /// `return` keyword
    Return,
}

const KEYWORDS: &[Token] = &[
    Token::As,
    Token::Const,
    Token::Fn,
    Token::If,
    Token::Else,
    Token::ElseIf,
    Token::And,
    Token::Or,
    Token::Import,
    Token::Let,
    Token::Type,
    Token::Opaque,
    Token::Pub,
    Token::Struct,
    Token::Enum,
    Token::Break,
    Token::Continue,
    Token::Async,
    Token::Await,
    Token::Return,
    // Total: 19
];

impl Token {
    pub fn is_keyword(&self) -> bool {
        KEYWORDS.contains(self)
    }

    pub fn try_from_keywords(word: &str) -> Option<Token> {
        match word {
            "const" => Some(Token::Const),
            "fn" => Some(Token::Fn),
            "if" => Some(Token::If),
            "else" => Some(Token::Else),
            "elseif" => Some(Token::ElseIf),
            "and" => Some(Token::And),
            "or" => Some(Token::Or),
            "import" => Some(Token::Import),
            "let" => Some(Token::Let),
            "type" => Some(Token::Type),
            "opaque" => Some(Token::Opaque),
            "pub" => Some(Token::Pub),
            "struct" => Some(Token::Struct),
            "enum" => Some(Token::Enum),
            "break" => Some(Token::Break),
            "continue" => Some(Token::Continue),
            "async" => Some(Token::Async),
            "return" => Some(Token::Return),
            _ => None,
        }
    }
}
