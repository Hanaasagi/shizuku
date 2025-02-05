#![feature(is_ascii_octdigit)]
#![allow(unused)]
pub mod ast;
pub mod lexer;
pub mod parser;
mod span;
mod token;

pub use lexer::Lexer;
pub use lexer::LexicalError;
pub use lexer::LexicalErrorType;
pub use span::SrcSpan;
pub use token::Base as NumberBase;
pub use token::Token;
