use std::collections::HashSet;

use crate::lexer::Tokenizer;

use hycc_ast::token::{Token, TokenGraph, TokenKind};
use hycc_diagnostic::DiagnosticContext;
use hycc_source::source::Source;
use hycc_util::ternary;

#[derive(Debug, Clone)]
pub enum TokenConsumptionKind {
    Absolute,
    Preserve,
    UponSuccess,
}

#[derive(Debug)]
pub struct Lexer<'s, 'd> {
    pub source: &'s Source,
    pub dctx: &'d mut DiagnosticContext,

    pub stream: Vec<TokenGraph>,
    offset: usize,
}

impl<'s, 'd> Lexer<'s, 'd> {
    pub fn new(source: &'s Source, dctx: &'d mut DiagnosticContext) -> Self {
        Self {
            source,
            dctx,
            stream: Vec::new(),
            offset: 0,
        }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.stream.len()
    }

    pub fn bsof(&self) -> bool {
        self.offset == 0
    }

    pub fn abs_eof(&self) -> bool {
        self.eof(true)
    }

    pub fn at_eof(&self) -> bool {
        self.eof(false)
    }

    pub fn eof(&self, absolute: bool) -> bool {
        self.offset >= (self.stream.len() - (1 + (!absolute as usize)))
    }

    pub fn adjust(&mut self) {
        self.adjustn(1)
    }

    pub fn adjustn(&mut self, n: usize) {
        self.offset += n
    }

    pub fn peek(&self) -> Option<&TokenGraph> {
        self.peekn(0)
    }

    pub fn peekn(&self, offset: usize) -> Option<&TokenGraph> {
        let pos = self.offset + offset;
        ternary!(pos >= self.len() - 1, None, self.stream.get(pos))
    }

    pub fn next(&mut self) -> Option<TokenGraph> {
        self.adjustn(1);
        Some(self.stream.get(self.offset - 1)?.clone())
    }

    pub fn skip_while(&mut self, mut predicate: impl FnMut(&TokenGraph) -> bool) {
        while let Some(token) = self.peek()
            && predicate(token)
        {
            self.adjust();
        }
    }

    pub fn skip_lf(&mut self) {
        self.skip_while(|token| {
            matches!(
                token,
                TokenGraph::Node(Token {
                    kind: TokenKind::LnFeed,
                    ..
                })
            )
        });
    }

    pub fn current(&self) -> &TokenGraph {
        self.stream
            .get((self.offset - 1).clamp(0, self.len() - 1))
            .unwrap()
    }

    pub fn expect(
        &mut self,
        kind: TokenKind,
        consumption: TokenConsumptionKind,
        exclude: Vec<TokenKind>,
    ) -> (bool, Option<TokenGraph>) {
        let set: HashSet<TokenKind> = exclude.into_iter().collect();

        // Skip excluded tokens whose kinds are within the token kind exclusion set
        let mut offset = 0;
        while let Some(TokenGraph::Node(token)) = self.peekn(offset) {
            if set.contains(&token.kind) {
                offset += 1;
            } else {
                break;
            }
        }

        let Some(tok_graph) = self.peekn(offset) else {
            return (false, None);
        };

        let tok_graph = tok_graph.clone();
        let matched = tok_graph.expect(kind);

        if matched && matches!(consumption, TokenConsumptionKind::UponSuccess) {
            self.adjustn(offset + 1);
        }

        (matched, Some(tok_graph))
    }

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

        std::mem::swap(&mut self.stream, &mut collection);
        self.stream.iter().for_each(|tg| println!("{tg}"));
    }
}
