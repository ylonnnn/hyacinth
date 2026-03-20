use std::fmt::Display;

use hycc_span::Span;

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
    pub fn len(&self) -> usize {
        self.span.len as usize
    }

    #[allow(non_snake_case)]
    pub fn Invalid(span: Span) -> Self {
        Self::new(TokenKind::Invalid, span)
    }

    pub fn view<'a>(&self, source: &'a String) -> &'a str {
        let offset = self.span.offset;
        &source[(offset as usize)..((offset + self.span.len as u32) as usize)]
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<{:?}:{}:{}>",
            self.kind,
            self.span.offset,
            self.span.offset + self.span.len as u32
        )
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum TokenKind {
    Int { base: u8 },
    Float { base: u8 },
    Bool,
    Char { terminated: bool },
    String { terminated: bool },

    Ident,
    MacroIdent,

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
                Self::Int { .. } => "int",
                Self::Float { .. } => "float",
                Self::Bool => "bool",
                Self::Char { .. } => "char",
                Self::String { .. } => "string",
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
