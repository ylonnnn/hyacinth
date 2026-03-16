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

    #[inline]
    pub fn len(&self) -> u32 {
        (self.span.end - self.span.start) + 1
    }

    #[allow(non_snake_case)]
    pub fn Invalid(span: Span) -> Self {
        Self {
            kind: TokenKind::Invalid,
            span,
        }
    }

    pub fn view<'a>(&self, source: &'a String) -> &'a str {
        &source[(self.span.start as usize)..(self.span.end as usize)]
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{:?}:{}:{}>", self.kind, self.span.start, self.span.end)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum TokenKind {
    Int { base: u32 },
    Float { base: u32 },
    Bool,
    Char { terminated: bool },
    String { terminated: bool },
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
    LessLess,
    Greater,
    GreaterGreater,
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
    ColonColon,
    Dot,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,

    // Miscellaneous
    LnFeed,
    DocComment,

    Invalid,
    Eof,
}

impl Default for TokenKind {
    fn default() -> Self {
        Self::Invalid
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Int { base: _ } => "int",
                Self::Float { base: _ } => "float",
                Self::Bool => "bool",
                Self::Char { terminated: _ } => "char",
                Self::String { terminated: _ } => "string",
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
                Self::ColonColon => "::",
                Self::Dot => ".",
                Self::LeftParen => "(",
                Self::RightParen => ")",
                Self::LeftBrace => "{",
                Self::RightBrace => "}",
                Self::LeftBracket => "[",
                Self::RightBracket => "]",

                // Miscellaneous
                Self::LnFeed => "\\n",
                Self::DocComment => "// doc-comment",

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
