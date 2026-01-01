use std::fs;

use crate::{
    core,
    syntax::{Token, Tokenizer},
    utils::control::terminate,
};

#[derive(Debug)]
pub struct Lexer {
    pub file: String,
    pub source: String,
    tokens: Vec<Token>,
}

#[derive(Debug)]
pub enum LexerSourceOrigin {
    File(String),
    Arbitrary(String),
}

pub type LexerResult = core::Result<()>;

impl Lexer {
    pub fn new(origin: LexerSourceOrigin) -> Self {
        match origin {
            LexerSourceOrigin::File(file) => Self {
                source: match fs::read_to_string(&file) {
                    Ok(content) => content,
                    Err(err) => terminate(&err.to_string()),
                },
                file,
                tokens: Vec::new(),
            },

            LexerSourceOrigin::Arbitrary(source) => Self {
                file: String::from("arbitrary.hyc"),
                source,
                tokens: Vec::new(),
            },
        }
    }

    pub fn tokenize(&mut self) -> LexerResult {
        let mut result = LexerResult::default();

        let mut tokenizer = Tokenizer::new(&self.source);
        let mut tk_res = tokenizer.tokenize();

        // Adapt the TokenizerResult and move the tokens to the Lexer
        result.adapt(&mut tk_res);

        if let Some(data) = tk_res.data.as_mut() {
            std::mem::swap(&mut self.tokens, data);
            self.tokens.iter().for_each(|token| println!("{token}"));
        }

        result
    }
}
