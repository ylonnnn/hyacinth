use std::fs;

use crate::{
    core::diagnostic::DiagnosticList,
    syntax::{Token, Tokenizer},
    utils::control::terminate,
};

#[derive(Debug)]
pub struct Lexer {
    pub path: String,
    pub source: String,
    pub diagnostics: DiagnosticList,

    tokens: Vec<Token>,
}

#[derive(Debug)]
pub enum LexerSourceOrigin {
    File(String),
    Arbitrary(String),
}

impl Lexer {
    pub fn new(origin: LexerSourceOrigin) -> Self {
        match origin {
            LexerSourceOrigin::File(path) => Self {
                source: match fs::read_to_string(&path) {
                    Ok(content) => content,
                    Err(err) => terminate(&err.to_string()),
                },
                path,
                tokens: Vec::new(),
                diagnostics: DiagnosticList::new(),
            },

            LexerSourceOrigin::Arbitrary(source) => Self {
                path: String::from("arbitrary.hyc"),
                source,
                tokens: Vec::new(),
                diagnostics: DiagnosticList::new(),
            },
        }
    }

    pub fn size(&self) -> usize {
        self.tokens.len()
    }

    pub fn tokenize(&mut self) {
        let mut tokenizer = Tokenizer::new(self);
        let mut tokens = tokenizer.tokenize();

        std::mem::swap(&mut self.tokens, &mut tokens);
        self.tokens.iter().for_each(|token| println!("{token}"));
    }
}
