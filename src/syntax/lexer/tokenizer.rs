use std::{collections::HashMap, u32};

use crate::{
    core::{self, Span},
    syntax::{Token, TokenKind},
};

pub struct Tokenizer<'a> {
    source: &'a str,
    offset: usize,
    reserved: HashMap<&'static str, TokenKind>,
}

pub type TokenizerResult = core::Result<Vec<Token>>;

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut inst = Self {
            source,
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
        self.offset >= self.source.len()
    }

    pub fn peek(&self) -> Option<char> {
        self.peekn(0)
    }

    pub fn peekn(&self, offset: usize) -> Option<char> {
        self.source.chars().nth(self.offset + offset)
    }

    pub fn next(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.consume(1);

        Some(c)
    }

    pub fn current(&self) -> Option<char> {
        let mut chars = self.source.chars();
        chars.nth(self.offset.clamp(1, self.source.len()) - 1)
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

    fn skip_until<F>(&mut self, mut predicate: F)
    where
        F: FnMut(char, usize) -> bool,
    {
        while let Some(c) = self.peek()
            && !predicate(c, self.offset)
        {
            self.consume(1);
        }
    }

    fn ignore_whitespace(&mut self, result: &mut TokenizerResult) {
        self.skip_until(|c, offset| {
            if c == '\n' {
                result.data.as_mut().map(|data| {
                    data.push(Token::new(TokenKind::LnFeed, (offset, offset + 1).into()))
                });
            }

            !c.is_whitespace()
        });
    }

    fn read_digits(&mut self, base: u32) {
        self.skip_until(|c, _| {
            // Validate each character
            // Failure will not stop lexing the characters

            if c.is_whitespace() {
                return true;
            }

            if !c.is_digit(base) {
                todo!(
                    "throw error: invalid numeric digit {} for numeric literals with base {}",
                    c,
                    base
                );
            }

            !c.is_alphanumeric()
        })
    }

    fn read_num(&mut self, _result: &mut TokenizerResult) -> Option<Token> {
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
            todo!("throw error: invalid numeric literal prefix")
        }

        self.read_digits(base); // Integral Part

        if self.expect('.') {
            self.read_digits(base); // Fractional Part

            Some(Token::new(
                TokenKind::Float(self.source[start..self.offset].to_string()),
                (start, self.offset).into(),
            ))
        } else {
            Some(Token::new(
                TokenKind::Int(self.source[start..self.offset].to_string()),
                (start, self.offset).into(),
            ))
        }
    }

    fn read_ident(&mut self) -> Token {
        let start = self.offset;

        self.skip_until(|c, _| c != '_' && !c.is_alphanumeric());

        let (ident, span) = (
            &self.source[start..self.offset],
            (start, self.offset).into(),
        );

        match self.reserved.get(&ident) {
            Some(kind) => Token::new(kind.clone(), span),
            _ => Token::new(TokenKind::Ident(ident.to_string()), span),
        }
    }

    pub fn tokenize(&mut self) -> TokenizerResult {
        let mut result: TokenizerResult = TokenizerResult::default();

        while !self.eof() {
            if let Some(c) = self.peek() {
                let start = self.offset;

                if c.is_whitespace() {
                    self.ignore_whitespace(&mut result);
                    continue;
                }

                let span: Span = (start, self.offset + 1).into();

                if let Some(token) = match c {
                    '_' | 'a'..'z' | 'A'..'Z' => Some(self.read_ident()),
                    '0'..'9' => self.read_num(&mut result),

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
                                    self.skip_until(|c, _| c == '\n');
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
                    if let Some(data) = result.data.as_mut() {
                        data.push(token);
                    }
                }
            }
        }

        result
    }
}
