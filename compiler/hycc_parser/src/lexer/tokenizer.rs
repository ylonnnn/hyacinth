use std::collections::HashMap;

use crate::{
    lexer::{Lexer, Token, TokenKind, token::TokenGraph},
    token,
};

use hycc_diagnostic::code::DiagnosticErrorKind;
use hycc_span::Span;
use hycc_util::{hashmap, is_ascii_digit, ternary};

pub struct Tokenizer<'l, 's, 'd> {
    lexer: &'l mut Lexer<'s, 'd>,
    offset: u32,
    reserved: HashMap<&'static str, TokenKind>,
}

impl<'l, 's, 'd> Tokenizer<'l, 's, 'd> {
    pub fn new(lexer: &'l mut Lexer<'s, 'd>) -> Self {
        Self {
            lexer,
            offset: 0,
            reserved: hashmap! {
                "fn" => TokenKind::Fn,
                "let" => TokenKind::Let,
                "true", "false" => TokenKind::Bool,
            },
        }
    }

    pub fn eof(&self) -> bool {
        self.offset >= self.lexer.source.data.len() as u32
    }

    pub fn peek(&self) -> Option<u8> {
        self.peekn(0)
    }

    pub fn peekn(&self, offset: u32) -> Option<u8> {
        self.lexer
            .source
            .data
            .bytes()
            .nth((self.offset + offset) as usize)
    }

    pub fn next(&mut self) -> Option<u8> {
        let c = self.peek()?;
        self.adjust();

        Some(c)
    }

    pub fn current(&self) -> Option<u8> {
        let src = &self.lexer.source.data;
        let mut bytes = src.bytes();

        bytes.nth((self.offset.clamp(1, src.len() as u32) - 1) as usize)
    }

    #[inline]
    pub const fn adjust(&mut self) {
        self.adjustn(1);
    }

    #[inline]
    pub const fn adjustn(&mut self, count: u32) {
        self.offset += count;
    }

    pub fn expect(&mut self, c: u8) -> bool {
        self.peek().is_some_and(|ch| {
            let eq = ch == c;
            self.adjustn(eq as u32);
            eq
        })
    }

    #[inline]
    fn skip_until<F>(&mut self, mut predicate: F) -> bool
    where
        F: FnMut(&mut Self, u8) -> bool,
    {
        while let Some(c) = self.peek() {
            if predicate(self, c) {
                return true;
            }

            self.adjust();
        }

        false
    }

    #[inline]
    fn ignore_whitespace(&mut self) -> Option<TokenGraph> {
        while let Some(c) = self.peek() {
            if !c.is_ascii_whitespace() {
                break;
            }

            self.adjust();
            if c == b'\n' {
                return Some(TokenGraph::Node(token!(
                    TokenKind::LnFeed,
                    (self.offset, 1, self.lexer.source.identifier.0).into()
                )));
            }
        }

        None
    }

    fn read_ident(&mut self) -> Token {
        let start = (self.offset, self.offset += 1).0;

        self.skip_until(|_, c| c != b'_' && !c.is_ascii_alphanumeric());

        let span = (
            start,
            (self.offset - start) as u16,
            self.lexer.source.identifier.0,
        )
            .into();

        let view = &self.lexer.source.data[(start as usize)..(self.offset as usize)];
        if view.starts_with('@') {
            token!(TokenKind::MacroIdent, span)
        } else {
            match self.reserved.get(view) {
                Some(kind) => token!(kind.clone(), span),
                _ => token!(TokenKind::Ident, span),
            }
        }
    }

    fn read_digits(&mut self, base: u32) {
        self.skip_until(|s, c| {
            if c.is_ascii_whitespace() || !c.is_ascii_alphanumeric() {
                return true;
            }

            if !is_ascii_digit(c, base) {
                s.lexer.dctx.error(
                    DiagnosticErrorKind::InvalidNumericLiteralDigit.into(),
                    &format!(
                        "invalid numeric digit `{}` for numeric literals with base `{}`",
                        c as char, base
                    ),
                    (s.offset, 1, self.lexer.source.identifier.0).into(),
                );
            }

            false
        });
    }

    fn read_num(&mut self) -> Option<Token> {
        let src_id = self.lexer.source.identifier.0;
        let start = self.offset;

        // Identify base according to prefix
        let mut base: u8 = 10;

        if let Some(b'0') = self.peek() {
            let c = self.peekn(1)?;
            #[allow(unused)]
            let mut n = 1; // Used but is not detected

            base = match c {
                b'b' => 2,
                b'o' => 8,
                b'x' => 16,
                _ if c.is_ascii_digit() || c.is_ascii_whitespace() || c == b'.' => 10,
                _ => {
                    n = c.is_ascii_alphabetic() as u32;
                    ternary!(n == 1, u8::MAX, 10)
                }
            };

            n = match base {
                2 | 8 | 16 => 2,
                _ => 1,
            };

            self.adjustn(n);
        }

        let source = &self.lexer.source.data[..];
        if base == u8::MAX {
            self.lexer.dctx.error(
                DiagnosticErrorKind::InvalidNumericLiteralPrefix.into(),
                &format!(
                    "invalid numeric literal prefix `{}`.",
                    &source[(start as usize)..=(self.offset as usize)]
                ),
                (start, 2, src_id).into(),
            );

            return None;
        }

        let _start = self.offset;
        self.read_digits(base as u32); // Integral Part

        // If the digit reading failed to proceed while the number is not
        // decimal (base != 10), there could be a dangling numeric literal prefix
        if _start == self.offset && base != 10 {
            self.lexer.dctx.error(
                DiagnosticErrorKind::InvalidNumericLiteralPrefix.into(),
                &format!(
                    "dangling numeric literal prefix `{}`.",
                    &source[(start as usize)..(self.offset as usize)]
                ),
                (start, (_start - start) as u16, src_id).into(),
            );
        }

        Some(if self.expect(b'.') {
            self.read_digits(base as u32); // Fractional Part

            token!(
                TokenKind::Float { base },
                (start, (self.offset - start) as u16, src_id).into()
            )
        } else {
            token!(
                TokenKind::Int { base },
                (start, (self.offset - start) as u16, src_id).into()
            )
        })
    }

    pub fn read_char_seq(&mut self) -> Option<Token> {
        let (start, quote) = (self.offset, self.next()?);
        let multi = quote == b'"';

        let mut seq_len = 0;
        let terminated = self.skip_until(|s, c| {
            let cmp = c == quote;
            if cmp {
                s.adjust();
                return cmp;
            }

            seq_len += 1;
            if c == b'\\' {
                s.adjust();
            }

            cmp
        });

        let diagnostics = &mut self.lexer.dctx;
        let span: Span = (
            start,
            (self.offset - start) as u16,
            self.lexer.source.identifier.0,
        )
            .into();

        if !terminated {
            diagnostics.error(
                DiagnosticErrorKind::UnterminatedCharacterSequence.into(),
                "unterminated character sequence.",
                span,
            );

            return None;
        }

        // String
        if multi {
            Some(token!(TokenKind::String { terminated }, span))
        }
        // Char
        else {
            if seq_len != 1 {
                diagnostics.error(
                    DiagnosticErrorKind::InvalidCharacterSequence.into(),
                    &format!(
                        "character sequence within `'` must contain exactly `{}` character, contains `{}`.",
                        1,
                        seq_len,
                    ),
                    span.clone(),
                );
            }

            Some(token!(TokenKind::Char { terminated }, span))
        }
    }

    pub fn tokenize_collection(
        &mut self,
        closing: u8,
        mut collection: Vec<TokenGraph>,
    ) -> TokenGraph {
        while let Some(c) = self.peek() {
            if let Some(tg) = self.tokenize() {
                collection.push(tg);
            }

            if c == closing {
                break;
            }
        }

        TokenGraph::Collection(collection)
    }

    #[inline]
    pub fn tokenize(&mut self) -> Option<TokenGraph> {
        let (src_id, _) = self.lexer.source.identifier;
        if self.eof() {
            return Some(TokenGraph::Node(token!(
                TokenKind::Eof,
                Span::new(self.offset, 1_u16, src_id)
            )));
        }

        let start = self.offset;
        let Some(c) = self.peek() else {
            unreachable!();
        };

        if c.is_ascii_whitespace() {
            return self.ignore_whitespace();
        }

        let span: Span = (start, 1_u16, src_id).into();

        // Collection
        let graph = match c {
            b'(' => {
                self.adjust();
                self.tokenize_collection(
                    b')',
                    vec![TokenGraph::Node(token!(TokenKind::LeftParen, span))],
                )
            }
            b'{' => {
                self.adjust();
                self.tokenize_collection(
                    b'}',
                    vec![TokenGraph::Node(token!(TokenKind::LeftBrace, span))],
                )
            }
            b'[' => {
                self.adjust();
                self.tokenize_collection(
                    b']',
                    vec![TokenGraph::Node(token!(TokenKind::LeftBracket, span))],
                )
            }

            _ => {
                let token = match c {
                    b'@' | b'_' | b'a'..=b'z' | b'A'..=b'Z' => Some(self.read_ident()),
                    b'0'..=b'9' => self.read_num(),
                    b'\'' | b'\"' => self.read_char_seq(),

                    b'+' => Some(match self.peekn(1) {
                        Some(b'+') => token!(TokenKind::PlusPlus, span.extend(1)),
                        _ => token!(TokenKind::Plus, span),
                    }),
                    b'-' => Some(token!(TokenKind::Minus, span)),
                    b'*' => Some(token!(TokenKind::Star, span)),
                    b'/' => match self.peekn(1) {
                        Some(b'/') => {
                            self.skip_until(|_, c| c == b'\n');
                            Some(token!(
                                TokenKind::DocComment,
                                (start, (self.offset - start) as u16, src_id).into()
                            ))
                        }
                        _ => Some(token!(TokenKind::Slash, span)),
                    },
                    b'%' => Some(token!(TokenKind::Percent, span)),
                    b'=' => Some(match self.peekn(1) {
                        Some(b'=') => token!(TokenKind::EqEq, span.extend(1)),
                        _ => token!(TokenKind::Eq, span),
                    }),
                    b'!' => Some(match self.peekn(1) {
                        Some(b'=') => token!(TokenKind::NotEq, span.extend(1)),
                        _ => token!(TokenKind::Not, span),
                    }),
                    b'<' => Some(match self.peekn(1) {
                        Some(b'<') => token!(TokenKind::LessLess, span.extend(1)),
                        Some(b'=') => token!(TokenKind::LessEq, span.extend(1)),
                        _ => token!(TokenKind::Less, span),
                    }),
                    b'>' => Some(match self.peekn(1) {
                        Some(b'>') => token!(TokenKind::GreaterGreater, span.extend(1)),
                        Some(b'=') => token!(TokenKind::GreaterEq, span.extend(1)),
                        _ => token!(TokenKind::Greater, span),
                    }),
                    b'&' => Some(match self.peekn(1) {
                        Some(b'&') => token!(TokenKind::AmpersandAmpersand, span.extend(1)),
                        _ => token!(TokenKind::Ampersand, span),
                    }),
                    b'|' => Some(match self.peekn(1) {
                        Some(b'|') => token!(TokenKind::PipePipe, span.extend(1)),
                        _ => token!(TokenKind::Pipe, span),
                    }),
                    b'.' => Some(token!(TokenKind::Dot, span)),
                    b',' => Some(token!(TokenKind::Comma, span)),
                    b';' => Some(token!(TokenKind::SemiColon, span)),
                    b':' => Some(match self.peekn(1) {
                        Some(b':') => token!(TokenKind::ColonColon, span.extend(1)),
                        _ => token!(TokenKind::Colon, span),
                    }),

                    b')' => Some(token!(TokenKind::RightParen, span)),
                    b'}' => Some(token!(TokenKind::RightBrace, span)),
                    b']' => Some(token!(TokenKind::RightBracket, span)),

                    inv => {
                        self.lexer.dctx.error(
                            DiagnosticErrorKind::UnknownCharacter.into(),
                            &format!("unknown character `{}`.", inv as char),
                            span.clone(),
                        );

                        Some(Token::Invalid(span))
                    }
                };

                let token = token?;
                self.offset = token.span.offset + token.len() as u32;

                TokenGraph::Node(token)
            }
        };

        Some(graph)
    }
}
