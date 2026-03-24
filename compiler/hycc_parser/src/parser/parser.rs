use hycc_ast::{
    Program,
    token::{TokenGraph, TokenKind},
    token_stream::{TokenConsumptionKind, TokenMatchExpectation, TokenStream},
};
use hycc_diagnostic::DiagnosticContext;
use hycc_source::Source;

use crate::{errors, parser::diag_ctx::ParserDiagCtx};

#[derive(Debug)]
pub struct Parser<'d, 's> {
    pub(super) stream: TokenStream,
    pub(super) dctx: ParserDiagCtx<'d>,
    pub(super) source: &'s Source,
}

impl<'d, 's> Parser<'d, 's> {
    pub fn new(source: &'s Source, dctx: ParserDiagCtx<'d>, stream: TokenStream) -> Self {
        Self {
            stream,
            dctx,
            source,
        }
    }

    pub fn peek_nonlf(&mut self) -> Option<&TokenGraph> {
        self.peekn_nonlf(0)
    }

    pub fn peekn_nonlf(&self, mut offset: usize) -> Option<&TokenGraph> {
        loop {
            let tg = self.stream.peekn(offset);
            let Some(tg) = tg else {
                break tg;
            };

            if tg.underlying()?.kind != TokenKind::LnFeed {
                break Some(tg);
            }

            offset += 1;
            continue;
        }
    }

    pub fn next_nonlf(&mut self) -> Option<TokenGraph> {
        let offset = self.stream.first_not_offset(vec![TokenKind::LnFeed]);
        self.stream.adjustn(offset);
        Some(self.stream.current().clone())
    }

    pub fn require(
        &mut self,
        kind: TokenKind,
        consumption: TokenConsumptionKind,
        exclude: Vec<TokenKind>,
        expectation: TokenMatchExpectation,
    ) -> Option<TokenGraph> {
        let (matched, tg) = self.stream.expect(kind, consumption, exclude, expectation);
        let Some(tg) = tg else { return None };

        if !matched {
            if let Some(tok) = tg.underlying() {
                self.dctx
                    .add(errors::token_kind_mismatch(self.source, tok, Some(kind)));
            }
            None
        } else {
            Some(tg)
        }
    }

    pub fn require_nonlf(
        &mut self,
        kind: TokenKind,
        consumption: TokenConsumptionKind,
        expectation: TokenMatchExpectation,
    ) -> Option<TokenGraph> {
        self.require(kind, consumption, vec![TokenKind::LnFeed], expectation)
    }

    pub fn require_exact_nonlf(&mut self, kind: TokenKind) -> Option<TokenGraph> {
        self.require(
            kind,
            TokenConsumptionKind::UponSuccess,
            vec![TokenKind::LnFeed],
            TokenMatchExpectation::Exact,
        )
    }

    pub fn require_similar_nonlf(&mut self, kind: TokenKind) -> Option<TokenGraph> {
        self.require(
            kind,
            TokenConsumptionKind::UponSuccess,
            vec![TokenKind::LnFeed],
            TokenMatchExpectation::Similar,
        )
    }

    pub fn use_stream<F, T>(&mut self, mut stream: TokenStream, mut f: F) -> T
    where
        F: FnMut(&mut Self) -> T,
    {
        std::mem::swap(&mut self.stream, &mut stream);
        let data = f(self);
        std::mem::swap(&mut self.stream, &mut stream);
        data
    }

    pub fn parse(&mut self) -> Program {
        let program = Program::new(Vec::new());

        while !self.stream.at_eof() {
            self.parse_item();
        }

        program
    }
}
