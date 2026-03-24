use std::{collections::HashSet, fmt::Display};

use hycc_util::ternary;

use crate::token::{Token, TokenGraph, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenConsumptionKind {
    Absolute,
    Preserve,
    UponSuccess,
}

#[derive(Debug, Clone)]
pub enum TokenMatchExpectation {
    Exact,
    Similar,
}

#[derive(Debug)]
pub struct TokenStream {
    data: Vec<TokenGraph>,
    offset: usize,
}

impl TokenStream {
    pub fn new(data: Vec<TokenGraph>) -> Self {
        Self { data, offset: 0 }
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
        self.offset >= (self.data.len() - (1 + (!absolute as usize)))
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
        ternary!(pos >= self.data.len() - 1, None, self.data.get(pos))
    }

    pub fn next(&mut self) -> Option<TokenGraph> {
        self.adjustn(1);
        Some(self.data.get(self.offset - 1)?.clone())
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
        self.data
            .get((self.offset - 1).clamp(0, self.data.len() - 1))
            .unwrap()
    }

    pub fn first_not_offset(&self, exclude: Vec<TokenKind>) -> usize {
        let set: HashSet<_> = exclude.into_iter().collect();
        let mut offset = 0;

        while let Some(TokenGraph::Node(token)) = self.peekn(offset) {
            if set.contains(&token.kind) {
                offset += 1;
            } else {
                break;
            }
        }

        offset
    }

    pub fn expect(
        &mut self,
        kind: TokenKind,
        consumption: TokenConsumptionKind,
        exclude: Vec<TokenKind>,
        expectation: TokenMatchExpectation,
    ) -> (bool, Option<TokenGraph>) {
        let offset = self.first_not_offset(exclude);
        let Some(tok_graph) = self.peekn(offset) else {
            return (false, None);
        };

        let tok_graph = tok_graph.clone();
        let matched = match expectation {
            TokenMatchExpectation::Exact => tok_graph.is(kind),
            TokenMatchExpectation::Similar => tok_graph.is_like(kind),
        };

        if matched && consumption == TokenConsumptionKind::UponSuccess {
            self.adjustn(offset + 1);
        }

        (matched, Some(tok_graph))
    }

    pub fn expect_exact(
        &mut self,
        kind: TokenKind,
        consumption: TokenConsumptionKind,
        exclude: Vec<TokenKind>,
    ) -> (bool, Option<TokenGraph>) {
        self.expect(kind, consumption, exclude, TokenMatchExpectation::Exact)
    }

    pub fn expect_similar(
        &mut self,
        kind: TokenKind,
        consumption: TokenConsumptionKind,
        exclude: Vec<TokenKind>,
    ) -> (bool, Option<TokenGraph>) {
        self.expect(kind, consumption, exclude, TokenMatchExpectation::Similar)
    }
}

impl Display for TokenStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for tg in &self.data {
            writeln!(f, "{tg}")?;
        }

        Ok(())
    }
}
