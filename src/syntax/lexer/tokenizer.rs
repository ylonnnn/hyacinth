use std::collections::HashMap;

use crate::{
    core::{Span, diagnostic::code::DiagnosticErrorKind},
    hashmap,
    syntax::{Lexer, Token, TokenKind},
    token,
};

pub struct Tokenizer<'a> {
    lexer: &'a mut Lexer,
    offset: usize,
    reserved: HashMap<&'static str, TokenKind>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(lexer: &'a mut Lexer) -> Self {
        Self {
            lexer,
            offset: 0,
            reserved: hashmap! {
                "let" => TokenKind::Let,
                "true", "false" => TokenKind::Bool,
            },
        }
    }

    pub fn eof(&self) -> bool {
        self.offset >= self.lexer.source.len()
    }

    pub fn peek(&self) -> Option<char> {
        self.peekn(0)
    }

    pub fn peekn(&self, offset: usize) -> Option<char> {
        self.lexer.source.chars().nth(self.offset + offset)
    }

    pub fn next(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.adjust(1);

        Some(c)
    }

    pub fn current(&self) -> Option<char> {
        let mut chars = self.lexer.source.chars();
        chars.nth(self.offset.clamp(1, self.lexer.source.len()) - 1)
    }

    pub fn adjust(&mut self, count: usize) {
        self.offset += count;
    }

    pub fn expect(&mut self, c: char) -> bool {
        self.peek().is_some_and(|ch| {
            let eq = ch == c;
            self.adjust(eq as usize);
            eq
        })
    }

    fn skip_until<F>(&mut self, mut predicate: F) -> bool
    where
        F: FnMut(&mut Self, char) -> bool,
    {
        while let Some(c) = self.peek() {
            if predicate(self, c) {
                return true;
            }

            self.adjust(c.len_utf8());
        }

        false
    }

    fn ignore_whitespace(&mut self, tokens: &mut Vec<Token>) {
        self.skip_until(|s, c| {
            if c == '\n' {
                tokens.push(token!(TokenKind::LnFeed, (s.offset, s.offset + 1).into()));
            }

            !c.is_whitespace()
        });
    }

    fn read_ident(&mut self) -> Token {
        let start = self.offset;

        self.skip_until(|_, c| c != '_' && !c.is_alphanumeric());

        let span = (start, self.offset).into();

        match self.reserved.get(&self.lexer.source[start..self.offset]) {
            Some(kind) => token!(kind.clone(), span),
            _ => token!(TokenKind::Ident, span),
        }
    }

    fn read_digits(&mut self, base: u32) {
        self.skip_until(|s, c| {
            if c.is_whitespace() || !c.is_alphanumeric() {
                return true;
            }

            if !c.is_digit(base) {
                s.lexer.diagnostics.error(
                    DiagnosticErrorKind::InvalidNumericLiteralDigit.into(),
                    &format!(
                        "invalid numeric digit `{}` for numeric literals with base `{}`",
                        c, base
                    ),
                    (s.offset, s.offset + 1).into(),
                );
            }

            false
        });
    }

    fn read_num(&mut self) -> Option<Token> {
        let start = self.offset;

        // Identify base according to prefix
        let mut base: u32 = 10;

        if let Some('0') = self.peek() {
            let c = self.peekn(1)?;

            base = match c {
                'b' => 2,
                'o' => 8,
                'x' => 16,
                _ if c.is_digit(10) || c.is_whitespace() || c == '.' => 10,
                _ => u32::MAX,
            };

            self.adjust(match base {
                2 | 8 | 16 => 2,
                _ => 1,
            });
        }

        if base == u32::MAX {
            self.lexer.diagnostics.error(
                DiagnosticErrorKind::InvalidNumericLiteralPrefix.into(),
                "invalid numeric literal prefix",
                (start, self.offset + 1).into(),
            );

            return None;
        }

        self.read_digits(base); // Integral Part

        if self.expect('.') {
            self.read_digits(base); // Fractional Part

            Some(token!(TokenKind::Float, (start, self.offset).into()))
        } else {
            Some(token!(TokenKind::Int, (start, self.offset).into()))
        }
    }

    pub fn read_char_seq(&mut self) -> Option<Token> {
        let (start, quote) = (self.offset, self.next()?);
        let multi = quote == '"';

        let mut seq_len = 0;
        let terminated = self.skip_until(|s, c| {
            let cmp = c == quote;
            if cmp {
                s.adjust(1);
                return cmp;
            }

            seq_len += 1;
            if c == '\\' {
                s.adjust(1);
            }

            cmp
        });

        let diagnostics = &mut self.lexer.diagnostics;
        let span: Span = (start, self.offset - 1).into();

        if !terminated {
            diagnostics.error(
                DiagnosticErrorKind::UnterminatedCharacterSequence.into(),
                "unterminated character sequence.",
                span,
            );

            return None;
        }

        // let sequence: String = self.lexer.source.chars().collect::<Vec<char>>()[start..self.offset]
        //     .into_iter()
        //     .collect();

        // String
        if multi {
            Some(token!(TokenKind::String, span))
        }
        // Char
        else {
            dbg!(&seq_len);

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

            Some(token!(TokenKind::Char, span))
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::<Token>::with_capacity(16);

        while !self.eof() {
            if let Some(c) = self.peek() {
                let start = self.offset;

                if c.is_whitespace() {
                    self.ignore_whitespace(&mut tokens);
                    continue;
                }

                let len = c.len_utf8();
                let span: Span = (start, self.offset + len).into();

                if let Some(token) = match c {
                    '_' | 'a'..='z' | 'A'..='Z' => Some(self.read_ident()),
                    '0'..='9' => self.read_num(),
                    '\'' | '\"' => self.read_char_seq(),

                    _ => {
                        let (token, consume) = match c {
                            '+' => match self.peekn(1) {
                                Some(ch) if ch == '+' => (
                                    Some(token!(TokenKind::PlusPlus, span.extend(ch.len_utf8()))),
                                    len + ch.len_utf8(),
                                ),
                                _ => (Some(token!(TokenKind::Plus, span)), len),
                            },
                            '-' => (Some(token!(TokenKind::Minus, span)), len),
                            '*' => (Some(token!(TokenKind::Star, span)), len),
                            '/' => match self.peekn(1) {
                                Some('/') => {
                                    self.skip_until(|_, c| c == '\n');
                                    (None, 0)
                                }
                                _ => (Some(token!(TokenKind::Slash, span)), len),
                            },
                            '%' => (Some(token!(TokenKind::Percent, span)), len),
                            '=' => match self.peekn(1) {
                                Some(ch) if ch == '=' => (
                                    Some(token!(TokenKind::EqEq, span.extend(ch.len_utf8()))),
                                    len + ch.len_utf8(),
                                ),
                                _ => (Some(token!(TokenKind::Eq, span)), len),
                            },
                            '!' => match self.peekn(1) {
                                Some('=') => (Some(token!(TokenKind::NotEq, span.extend(1))), 2),
                                _ => (Some(token!(TokenKind::Not, span)), len),
                            },
                            '<' => match self.peekn(1) {
                                Some('=') => (Some(token!(TokenKind::LessEq, span.extend(1))), 2),
                                _ => (Some(token!(TokenKind::Less, span)), len),
                            },
                            '>' => match self.peekn(1) {
                                Some(ch) if ch == '=' => (
                                    Some(token!(TokenKind::GreaterEq, span.extend(ch.len_utf8()))),
                                    len + ch.len_utf8(),
                                ),
                                _ => (Some(token!(TokenKind::Greater, span)), len),
                            },
                            '&' => match self.peekn(1) {
                                Some(ch) if ch == '&' => (
                                    Some(token!(
                                        TokenKind::AmpersandAmpersand,
                                        span.extend(ch.len_utf8())
                                    )),
                                    len + ch.len_utf8(),
                                ),
                                _ => (Some(token!(TokenKind::Ampersand, span)), len),
                            },

                            '|' => match self.peekn(1) {
                                Some(ch) if ch == '|' => (
                                    Some(token!(TokenKind::PipePipe, span.extend(ch.len_utf8()))),
                                    len + ch.len_utf8(),
                                ),
                                _ => (Some(token!(TokenKind::Pipe, span)), len),
                            },

                            ',' => (Some(token!(TokenKind::Comma, span)), len),
                            ';' => (Some(token!(TokenKind::SemiColon, span)), len),
                            ':' => (Some(token!(TokenKind::Colon, span)), len),
                            '.' => (Some(token!(TokenKind::Dot, span)), len),
                            '(' => (Some(token!(TokenKind::LeftParen, span)), len),
                            ')' => (Some(token!(TokenKind::RightParen, span)), len),
                            '{' => (Some(token!(TokenKind::LeftBrace, span)), len),
                            '}' => (Some(token!(TokenKind::RightBrace, span)), len),
                            '[' => (Some(token!(TokenKind::LeftBracket, span)), len),
                            ']' => (Some(token!(TokenKind::RightBracket, span)), len),
                            inv => {
                                self.lexer.diagnostics.error(
                                    DiagnosticErrorKind::UnknownCharacter.into(),
                                    &format!("unknown character `{}`.", inv),
                                    span.clone(),
                                );

                                (Some(Token::Invalid(span)), len)
                            }
                        };

                        self.adjust(consume);
                        token
                    }
                } {
                    tokens.push(token);
                }
            }
        }

        tokens.push(token!(
            TokenKind::Eof,
            (self.offset, self.offset + 1).into()
        ));

        tokens
    }
}
