use std::{collections::HashSet, fs};

use crate::{
    core::diagnostic::DiagnosticList,
    syntax::{Token, TokenKind, Tokenizer},
    ternary,
    utils::control::terminate,
};

#[derive(Debug)]
pub struct Lexer {
    pub path: String,
    pub source: String,
    pub diagnostics: DiagnosticList,

    tokens: Vec<Token>,
    offset: usize,
}

#[derive(Debug)]
pub enum LexerSourceOrigin {
    File(String),
    Arbitrary(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenConsumptionType {
    Absolute,
    Preserve,
    UponSuccess,
}

impl Lexer {
    pub fn new(origin: LexerSourceOrigin) -> Self {
        let mut inst = Self {
            path: String::new(),
            source: String::new(),
            diagnostics: DiagnosticList::new(),
            tokens: Vec::new(),
            offset: 0,
        };

        match origin {
            LexerSourceOrigin::File(path) => {
                inst.source = match fs::read_to_string(&path) {
                    Ok(content) => content,
                    Err(err) => terminate(&err.to_string()),
                };

                inst.path = path;
            }

            LexerSourceOrigin::Arbitrary(source) => {
                inst.path = String::from("arbitrary.hyc");
                inst.source = source;
            }
        };

        inst
    }

    pub fn size(&self) -> usize {
        self.tokens.len()
    }

    pub fn bsof(&self) -> bool {
        self.offset == 0
    }

    pub fn abs_eof(&self) -> bool {
        self.eof(true)
    }

    pub fn eof(&self, absolute: bool) -> bool {
        self.offset >= (self.size() - (1 + (!absolute as usize)))
    }

    pub fn peek(&self) -> Option<&Token> {
        self.peekn(0)
    }

    pub fn peekn(&self, offset: usize) -> Option<&Token> {
        let pos = self.offset + offset;
        ternary!(pos >= self.size() - 1, None, self.tokens.get(pos))
    }

    pub fn next(&mut self) -> Option<Token> {
        self.consume(1);
        Some(self.tokens.get(self.offset - 1)?.clone())
    }

    pub fn skip_while(&mut self, mut predicate: impl FnMut(&Token) -> bool) {
        while let Some(token) = self.peek()
            && predicate(token)
        {
            self.consume(1);
        }
    }

    pub fn skip_lf(&mut self) {
        self.skip_while(|token| token.kind == TokenKind::LnFeed);
    }

    pub fn current(&self) -> &Token {
        &self.tokens[(self.offset - 1).clamp(0, self.size() - 1)]
    }

    pub fn consume(&mut self, offset: usize) {
        self.offset += offset
    }

    pub fn expect(
        &mut self,
        kind: TokenKind,
        consumption: TokenConsumptionType,
        exclude: Vec<TokenKind>,
    ) -> Option<Token> {
        let set: HashSet<TokenKind> = exclude.into_iter().collect();

        let mut offset = 0;
        while let Some(token) = self.peekn(offset) {
            if !set.contains(&token.kind) {
                break;
            }

            offset += 1;
        }

        let Some(token) = self.peekn(offset) else {
            return None;
        };

        let token = token.clone();
        if token.kind != kind {
            return None;
        }

        if consumption == TokenConsumptionType::UponSuccess {
            self.consume(offset + 1);
        }

        Some(token)
    }

    pub fn tokenize(&mut self) {
        let mut tokenizer = Tokenizer::new(self);
        let mut tokens = tokenizer.tokenize();

        std::mem::swap(&mut self.tokens, &mut tokens);
        self.tokens.iter().for_each(|token| println!("{token}"));
    }
}
