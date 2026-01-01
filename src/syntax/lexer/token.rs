use std::fmt;

use crate::core::Span;

#[derive(Debug)]
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
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{:?}:{}:{}>", self.kind, self.span.start, self.span.end)
    }
}

#[derive(Debug, Clone)]
pub enum TokenKind {
    Int(String),
    Float(String),
    Bool(String),
    Ident(String),

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

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Int(x) | Self::Float(x) | Self::Bool(x) | Self::Ident(x) => x.clone(),

                Self::Let => "let".to_owned(),

                Self::Plus => "+".to_owned(),
                Self::PlusPlus => "++".to_owned(),
                Self::Minus => "-".to_owned(),
                Self::Star => "*".to_owned(),
                Self::Slash => "/".to_owned(),
                Self::Percent => "%".to_owned(),
                Self::Eq => "=".to_owned(),
                Self::EqEq => "==".to_owned(),
                Self::NotEq => "!=".to_owned(),
                Self::Less => "<".to_owned(),
                Self::Greater => ">".to_owned(),
                Self::LessEq => "<=".to_owned(),
                Self::GreaterEq => ">=".to_owned(),
                Self::Ampersand => "&".to_owned(),
                Self::AmpersandAmpersand => "&&".to_owned(),
                Self::Pipe => "|".to_owned(),
                Self::PipePipe => "||".to_owned(),
                Self::Not => "!".to_owned(),

                // Delimiters
                Self::Comma => ",".to_owned(),
                Self::SemiColon => ";".to_owned(),
                Self::Colon => ":".to_owned(),
                Self::Dot => ".".to_owned(),
                Self::LeftParen => "(".to_owned(),
                Self::RightParen => ")".to_owned(),
                Self::LeftBrace => "{".to_owned(),
                Self::RightBrace => "}".to_owned(),
                Self::LeftBracket => "[".to_owned(),
                Self::RightBracket => "]".to_owned(),

                // Miscellaneous
                Self::LnFeed => "\\n".to_owned(),

                Self::Invalid => "Invalid".to_owned(),
                Self::Eof => "EOF".to_owned(),

                #[allow(unreachable_patterns)] // For Future Token Kinds
                _ => "Unknown".to_owned(),
            }
        )
    }
}
