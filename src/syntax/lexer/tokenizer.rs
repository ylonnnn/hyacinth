use std::collections::HashMap;

use crate::{
    core::{Span, diagnostic::code::DiagnosticErrorKind},
    syntax::{Lexer, Token, TokenKind},
};

pub struct Tokenizer<'a> {
    lexer: &'a mut Lexer,
    offset: usize,
    reserved: HashMap<&'static str, TokenKind>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(lexer: &'a mut Lexer) -> Self {
        let mut inst = Self {
            lexer,
            offset: 0,
            reserved: HashMap::new(),
        };

        inst.initialize();

        inst
    }

    pub fn initialize(&mut self) {
        self.reserved.insert("let", TokenKind::Let);
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
        self.consume(1);

        Some(c)
    }

    pub fn current(&self) -> Option<char> {
        let mut chars = self.lexer.source.chars();
        chars.nth(self.offset.clamp(1, self.lexer.source.len()) - 1)
    }

    pub fn consume(&mut self, count: usize) {
        self.offset += count;
    }

    pub fn expect(&mut self, c: char) -> bool {
        self.peek().is_some_and(|ch| {
            let eq = ch == c;
            self.consume(eq as usize);
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

            self.consume(1);
        }

        false
    }

    fn ignore_whitespace(&mut self, tokens: &mut Vec<Token>) {
        self.skip_until(|s, c| {
            if c == '\n' {
                tokens.push(Token::new(
                    TokenKind::LnFeed,
                    (s.offset, s.offset + 1).into(),
                ))
            }

            !c.is_whitespace()
        });
    }

    fn read_ident(&mut self) -> Token {
        let start = self.offset;

        self.skip_until(|_, c| c != '_' && !c.is_alphanumeric());

        let (ident, span) = (
            &self.lexer.source[start..self.offset],
            (start, self.offset).into(),
        );

        match self.reserved.get(&ident) {
            Some(kind) => Token::new(kind.clone(), span),
            _ => Token::new(TokenKind::Ident(ident.to_string()), span),
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
                _ if c.is_digit(10) => 10,
                _ => u32::MAX,
            };

            self.consume(match base {
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
        }

        self.read_digits(base); // Integral Part

        if self.expect('.') {
            self.read_digits(base); // Fractional Part

            Some(Token::new(
                TokenKind::Float(self.lexer.source[start..self.offset].to_string()),
                (start, self.offset).into(),
            ))
        } else {
            Some(Token::new(
                TokenKind::Int(self.lexer.source[start..self.offset].to_string()),
                (start, self.offset).into(),
            ))
        }
    }

    pub fn read_char_seq(&mut self) -> Option<Token> {
        let (start, quote) = (self.offset, self.next()?);
        let multi = quote == '"';

        let terminated = self.skip_until(|s, c| {
            if c == '\\' {
                s.consume(1);
                return false;
            }

            let cmp = c == quote;
            if cmp {
                s.consume(1);
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

        let sequence: String = self.lexer.source.chars().collect::<Vec<char>>()[start..self.offset]
            .into_iter()
            .collect();

        println!("{sequence} | len: {}", sequence.len());

        let seq_len = sequence.len() - 2; // 2 for quotation marks

        // String
        if multi {
            Some(Token::new(TokenKind::String(sequence), span))
        }
        // Char
        else {
            if seq_len > 1 {
                diagnostics.error(
                    DiagnosticErrorKind::InvalidCharacterSequence.into(),
                    &format!(
                        "character sequence within `'` cannot contain more than `{}` character.",
                        1
                    ),
                    span,
                );
            }

            Some(Token::new(TokenKind::Char(sequence), span))
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

                let span: Span = (start, self.offset + 1).into();

                if let Some(token) = match c {
                    '_' | 'a'..='z' | 'A'..='Z' => Some(self.read_ident()),
                    '0'..='9' => self.read_num(),
                    '\'' | '\"' => self.read_char_seq(),

                    _ => {
                        let (token, consume) = match c {
                            '+' => match self.peekn(1) {
                                Some('+') => {
                                    (Some(Token::new(TokenKind::PlusPlus, span.extend(1))), 2)
                                }
                                _ => (Some(Token::new(TokenKind::Plus, span)), 1),
                            },
                            '-' => (Some(Token::new(TokenKind::Minus, span)), 1),
                            '*' => (Some(Token::new(TokenKind::Star, span)), 1),
                            '/' => match self.peekn(1) {
                                Some('/') => {
                                    self.skip_until(|_, c| c == '\n');
                                    (None, 0)
                                }
                                _ => (Some(Token::new(TokenKind::Slash, span)), 1),
                            },
                            '%' => (Some(Token::new(TokenKind::Percent, span)), 1),
                            '=' => match self.peekn(1) {
                                Some('=') => (Some(Token::new(TokenKind::EqEq, span.extend(1))), 2),
                                _ => (Some(Token::new(TokenKind::Eq, span)), 1),
                            },
                            '!' => match self.peekn(1) {
                                Some('=') => {
                                    (Some(Token::new(TokenKind::NotEq, span.extend(1))), 2)
                                }
                                _ => (Some(Token::new(TokenKind::Not, span)), 1),
                            },
                            '<' => match self.peekn(1) {
                                Some('=') => {
                                    (Some(Token::new(TokenKind::LessEq, span.extend(1))), 2)
                                }
                                _ => (Some(Token::new(TokenKind::Less, span)), 1),
                            },
                            '>' => match self.peekn(1) {
                                Some('=') => {
                                    (Some(Token::new(TokenKind::GreaterEq, span.extend(1))), 2)
                                }
                                _ => (Some(Token::new(TokenKind::Greater, span)), 1),
                            },
                            '&' => match self.peekn(1) {
                                Some('&') => (
                                    Some(Token::new(TokenKind::AmpersandAmpersand, span.extend(1))),
                                    2,
                                ),
                                _ => (Some(Token::new(TokenKind::Ampersand, span)), 1),
                            },

                            '|' => match self.peekn(1) {
                                Some('|') => {
                                    (Some(Token::new(TokenKind::PipePipe, span.extend(1))), 2)
                                }
                                _ => (Some(Token::new(TokenKind::Pipe, span)), 1),
                            },

                            ',' => (Some(Token::new(TokenKind::Comma, span)), 1),
                            ';' => (Some(Token::new(TokenKind::SemiColon, span)), 1),
                            ':' => (Some(Token::new(TokenKind::Colon, span)), 1),
                            '.' => (Some(Token::new(TokenKind::Dot, span)), 1),
                            '(' => (Some(Token::new(TokenKind::LeftParen, span)), 1),
                            ')' => (Some(Token::new(TokenKind::RightParen, span)), 1),
                            '{' => (Some(Token::new(TokenKind::LeftBrace, span)), 1),
                            '}' => (Some(Token::new(TokenKind::RightBrace, span)), 1),
                            '[' => (Some(Token::new(TokenKind::LeftBracket, span)), 1),
                            ']' => (Some(Token::new(TokenKind::RightBracket, span)), 1),
                            _ => (Some(Token::Invalid(span)), 1),
                        };

                        self.consume(consume);
                        token
                    }
                } {
                    tokens.push(token);
                }
            }
        }

        tokens.push(Token::new(
            TokenKind::Eof,
            (self.offset, self.offset + 1).into(),
        ));

        tokens
    }
}
