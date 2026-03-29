use std::fmt::{Debug, Display};

use hycc_span::Span;
use hycc_util::ternary;

#[derive(Clone)]
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
        ternary!(
            self.kind != TokenKind::Eof,
            &source[(offset as usize)..((offset + self.span.len as u32) as usize)],
            "EOF"
        )
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
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

#[derive(Debug, Clone)]
pub enum TokenGraph {
    Node(Token),
    Collection { data: Vec<TokenGraph>, eof: bool },
}

impl TokenGraph {
    pub fn underlying(&self) -> Option<&Token> {
        match self {
            Self::Node(token) => Some(token),
            Self::Collection { data, .. } => data.get(0)?.underlying(),
        }
    }

    pub fn is(&self, kind: TokenKind) -> bool {
        match self.underlying() {
            Some(tok) => tok.kind == kind,
            _ => false,
        }
    }

    pub fn is_like(&self, kind: TokenKind) -> bool {
        use std::mem::discriminant;
        match self.underlying() {
            Some(tok) => discriminant(&tok.kind) == discriminant(&kind),
            _ => false,
        }
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        match self {
            Self::Node(token) => write!(f, "{token}"),
            Self::Collection { data, .. } => {
                write!(f, "[\n")?;
                for tg in data {
                    write!(f, "{: <indent$}", "", indent = (indent + 1) * 4)?;
                    tg.fmt_with_indent(f, indent + 1)?;
                    writeln!(f)?;
                }

                write!(f, "{: <indent$}]", "", indent = indent * 4)
            }
        }
    }
}

impl Display for TokenGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_with_indent(f, 0)
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TokenIdentKind {
    Normal,

    // Keywords
    Fn,
    Let,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TokenKind {
    Int { base: u8 },
    Float { base: u8 },
    Bool,
    Char { terminated: bool },
    String { terminated: bool },

    Ident(TokenIdentKind),
    MacroIdent,

    // Operators
    Plus,
    PlusPlus,
    PlusEq,
    Minus,
    MinusMinus,
    MinusEq,
    Star,
    StarEq,
    Slash,
    SlashEq,
    Percent,
    PercentEq,
    Eq,
    EqEq,
    BangEq,
    Less,
    LessEq,
    LessLess,
    LessMinus,
    Greater,
    GreaterEq,
    GreaterGreater,
    MinusGreater,
    Ampersand,
    AmpersandAmpersand,
    Pipe,
    PipePipe,
    Bang,

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
                Self::Ident(kind) => match kind {
                    TokenIdentKind::Fn => "fn",
                    TokenIdentKind::Let => "let",
                    _ => "ident",
                },

                Self::MacroIdent => "macro ident",

                Self::Plus => "+",
                Self::PlusPlus => "++",
                Self::PlusEq => "+=",
                Self::Minus => "-",
                Self::MinusMinus => "--",
                Self::MinusEq => "-=",
                Self::Star => "*",
                Self::StarEq => "*=",
                Self::Slash => "/",
                Self::SlashEq => "/=",
                Self::Percent => "%",
                Self::PercentEq => "%=",
                Self::Eq => "=",
                Self::EqEq => "==",
                Self::BangEq => "!=",
                Self::Less => "<",
                Self::LessEq => "<=",
                Self::LessLess => "<<",
                Self::LessMinus => "<-",
                Self::Greater => ">",
                Self::GreaterEq => ">=",
                Self::GreaterGreater => ">>",
                Self::MinusGreater => "->",
                Self::Ampersand => "&",
                Self::AmpersandAmpersand => "&&",
                Self::Pipe => "|",
                Self::PipePipe => "||",
                Self::Bang => "!",

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
