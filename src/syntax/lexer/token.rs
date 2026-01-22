use std::fmt::Display;

use crate::core::Span;

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    #[allow(non_snake_case)]
    pub fn Invalid(span: Span) -> Self {
        Self {
            kind: TokenKind::Invalid,
            span,
        }
    }

    pub fn view<'a>(&self, source: &'a String) -> &'a str {
        &source[self.span.start..self.span.end]
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{:?}:{}:{}>", self.kind, self.span.start, self.span.end)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum TokenKind {
    Int,
    Float,
    Bool,
    Char,
    String,
    Ident,

    Let,

    // Operators
    Plus,
    PlusPlus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    EqEq,
    NotEq,
    Less,
    Greater,
    LessEq,
    GreaterEq,
    Ampersand,
    AmpersandAmpersand,
    Pipe,
    PipePipe,
    Not,

    // Delimiters
    Comma,
    SemiColon,
    Colon,
    Dot,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,

    // Miscellaneous
    LnFeed,

    Invalid,
    Eof,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Int => "int",
                Self::Float => "float",
                Self::Bool => "bool",
                Self::Char => "char",
                Self::String => "string",
                Self::Ident => "ident",

                Self::Let => "let",

                Self::Plus => "+",
                Self::PlusPlus => "++",
                Self::Minus => "-",
                Self::Star => "*",
                Self::Slash => "/",
                Self::Percent => "%",
                Self::Eq => "=",
                Self::EqEq => "==",
                Self::NotEq => "!=",
                Self::Less => "<",
                Self::Greater => ">",
                Self::LessEq => "<=",
                Self::GreaterEq => ">=",
                Self::Ampersand => "&",
                Self::AmpersandAmpersand => "&&",
                Self::Pipe => "|",
                Self::PipePipe => "||",
                Self::Not => "!",

                // Delimiters
                Self::Comma => ",",
                Self::SemiColon => ";",
                Self::Colon => ":",
                Self::Dot => ".",
                Self::LeftParen => "(",
                Self::RightParen => ")",
                Self::LeftBrace => "{",
                Self::RightBrace => "}",
                Self::LeftBracket => "[",
                Self::RightBracket => "]",

                // Miscellaneous
                Self::LnFeed => "\\n",

                Self::Invalid => "Invalid",
                Self::Eof => "EOF",

                #[allow(unreachable_patterns)] // For Future Token Kinds
                _ => "Unknown",
            }
        )
    }
}

#[macro_export]
macro_rules! token {
    ($k:expr, $sp:expr) => {
        Token::new($k, $sp)
    };
}
