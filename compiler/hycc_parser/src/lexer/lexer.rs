use crate::lexer::{TokenKind, Tokenizer, token::TokenGraph};

use hycc_diagnostic::DiagnosticContext;
use hycc_source::source::Source;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenConsumptionType {
    Absolute,
    Preserve,
    UponSuccess,
}

#[derive(Debug)]
pub struct Lexer<'s, 'd> {
    pub source: &'s Source,
    pub dctx: &'d mut DiagnosticContext,

    pub token_graph: TokenGraph,
    offset: usize,
}

impl<'s, 'd> Lexer<'s, 'd> {
    pub fn new(source: &'s Source, dctx: &'d mut DiagnosticContext) -> Self {
        Self {
            source,
            dctx,
            token_graph: TokenGraph::Collection {
                data: Vec::new(),
                eof: false,
            },
            offset: 0,
        }
    }

    // pub fn len(&self) -> usize {
    //     self.tokens.len()
    // }

    // pub fn bsof(&self) -> bool {
    //     self.offset == 0
    // }

    // pub fn abs_eof(&self) -> bool {
    //     self.eof(true)
    // }

    // pub fn at_eof(&self) -> bool {
    //     self.eof(false)
    // }

    // pub fn eof(&self, absolute: bool) -> bool {
    //     self.offset >= (self.len() - (1 + (!absolute as usize)))
    // }

    // pub fn peek(&self) -> Option<&Token> {
    //     self.peekn(0)
    // }

    // pub fn peekn(&self, offset: usize) -> Option<&Token> {
    //     let pos = self.offset + offset;
    //     ternary!(pos >= self.len() - 1, None, self.tokens.get(pos))
    // }

    // pub fn next(&mut self) -> Option<Token> {
    //     self.consume(1);
    //     Some(self.tokens.get(self.offset - 1)?.clone())
    // }

    // pub fn skip_while(&mut self, mut predicate: impl FnMut(&Token) -> bool) {
    //     while let Some(token) = self.peek()
    //         && predicate(token)
    //     {
    //         self.consume(1);
    //     }
    // }

    // pub fn skip_lf(&mut self) {
    //     self.skip_while(|token| token.kind == TokenKind::LnFeed);
    // }

    // pub fn current(&self) -> &Token {
    //     &self.tokens[(self.offset - 1).clamp(0, self.len() - 1)]
    // }

    // pub fn consume(&mut self, offset: usize) {
    //     self.offset += offset
    // }

    // pub fn expect(
    //     &mut self,
    //     kind: TokenKind,
    //     consumption: TokenConsumptionType,
    //     exclude: Vec<TokenKind>,
    // ) -> (Option<Token>, bool) {
    //     let set: HashSet<TokenKind> = exclude.into_iter().collect();

    //     let mut offset = 0;
    //     while let Some(token) = self.peekn(offset) {
    //         if !set.contains(&token.kind) {
    //             break;
    //         }

    //         offset += 1;
    //     }

    //     let Some(token) = self.peekn(offset) else {
    //         return (None, false);
    //     };

    //     let token = token.clone();
    //     if token.kind != kind {
    //         return (Some(token), false);
    //     }

    //     if consumption == TokenConsumptionType::UponSuccess {
    //         self.consume(offset + 1);
    //     }

    //     (Some(token.clone()), true)
    // }

    pub fn tokenize(&mut self) {
        let mut tokenizer = Tokenizer::new(self);
        let mut collection = Vec::new();

        let mut terminate = false;

        while !terminate {
            let Some(tg) = tokenizer.tokenize() else {
                continue;
            };

            if let TokenGraph::Node(token) = &tg
                && matches!(token.kind, TokenKind::Eof)
            {
                terminate = true;
            }

            collection.push(tg);
        }

        let TokenGraph::Collection { data, .. } = &mut self.token_graph else {
            unreachable!()
        };

        std::mem::swap(data, &mut collection);
        dbg!(data);
        // self.tokens.iter().for_each(|token| println!("{token}"));
    }
}
