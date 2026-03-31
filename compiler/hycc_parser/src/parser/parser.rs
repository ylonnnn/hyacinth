use hycc_ast::{
    Program,
    token::{Token, TokenGraph, TokenKind},
    token_stream::{TokenConsumptionKind, TokenMatchExpectation, TokenStream},
};
use hycc_diagnostic::DiagnosticContext;
use hycc_source::Source;

use crate::{errors, parser::diag_ctx::ParserDiagCtx};

#[derive(Debug)]
pub struct Parser<'d, 's> {
    pub(super) stream: TokenStream,
    pub dctx: ParserDiagCtx<'d>,
    pub(super) source: &'s Source,

    pub(super) generic_delimeter_encounters: usize,
}

pub type ParseResult<T, E = bool> = Result<T, E>;

impl<'d, 's> Parser<'d, 's> {
    pub fn new(source: &'s Source, dctx: ParserDiagCtx<'d>, stream: TokenStream) -> Self {
        Self {
            stream,
            dctx,
            source,

            generic_delimeter_encounters: 0,
        }
    }

    pub fn adjust_to_nonlf(&mut self) {
        self.stream
            .adjustn(self.stream.first_not_offset(vec![TokenKind::LnFeed]));
    }

    pub fn peek_nonlf(&self) -> Option<&TokenGraph> {
        self.peekn_nonlf(0)
    }

    pub fn peekn_nonlf(&self, mut offset: usize) -> Option<&TokenGraph> {
        while let Some(tg) = self.stream.peekn(offset) {
            let Some(tok) = tg.underlying() else {
                return None;
            };

            if tok.kind != TokenKind::LnFeed {
                return Some(tg);
            }

            offset += 1
        }

        None
    }

    pub fn peek_nonlf_token(&self) -> Option<&Token> {
        self.peek_nonlf()?.underlying()
    }

    pub fn next_nonlf(&mut self) -> Option<TokenGraph> {
        let offset = self.stream.first_not_offset(vec![TokenKind::LnFeed]) + 1;
        self.stream.adjustn(offset);
        Some(self.stream.current().clone())
    }

    pub fn next_nonlf_token(&mut self) -> Option<Token> {
        Some(self.next_nonlf()?.underlying()?.clone())
    }

    pub fn expect_nonlf(
        &mut self,
        kind: TokenKind,
        consumption: TokenConsumptionKind,
        expectation: TokenMatchExpectation,
    ) -> (bool, Option<TokenGraph>) {
        self.stream
            .expect(kind, consumption, vec![TokenKind::LnFeed], expectation)
    }

    pub fn expect_exact_nonlf(&mut self, kind: TokenKind) -> (bool, Option<TokenGraph>) {
        self.expect_nonlf(
            kind,
            TokenConsumptionKind::UponSuccess,
            TokenMatchExpectation::Exact,
        )
    }

    pub fn expect_preserved_exact_nonlf(&mut self, kind: TokenKind) -> (bool, Option<TokenGraph>) {
        self.expect_nonlf(
            kind,
            TokenConsumptionKind::Preserve,
            TokenMatchExpectation::Exact,
        )
    }

    pub fn expect_abs_exact_nonlf(&mut self, kind: TokenKind) -> (bool, Option<TokenGraph>) {
        self.expect_nonlf(
            kind,
            TokenConsumptionKind::Absolute,
            TokenMatchExpectation::Exact,
        )
    }

    pub fn expect_similar_nonlf(&mut self, kind: TokenKind) -> (bool, Option<TokenGraph>) {
        self.expect_nonlf(
            kind,
            TokenConsumptionKind::UponSuccess,
            TokenMatchExpectation::Similar,
        )
    }

    pub fn expect_preserved_similar_nonlf(
        &mut self,
        kind: TokenKind,
    ) -> (bool, Option<TokenGraph>) {
        self.expect_nonlf(
            kind,
            TokenConsumptionKind::Preserve,
            TokenMatchExpectation::Similar,
        )
    }

    pub fn expect_abs_similar_nonlf(&mut self, kind: TokenKind) -> (bool, Option<TokenGraph>) {
        self.expect_nonlf(
            kind,
            TokenConsumptionKind::Absolute,
            TokenMatchExpectation::Similar,
        )
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
        self.require_nonlf(
            kind,
            TokenConsumptionKind::UponSuccess,
            TokenMatchExpectation::Exact,
        )
    }

    pub fn require_similar_nonlf(&mut self, kind: TokenKind) -> Option<TokenGraph> {
        self.require_nonlf(
            kind,
            TokenConsumptionKind::UponSuccess,
            TokenMatchExpectation::Similar,
        )
    }

    pub fn require_abs_exact_nonlf(&mut self, kind: TokenKind) -> Option<TokenGraph> {
        self.require_nonlf(
            kind,
            TokenConsumptionKind::Absolute,
            TokenMatchExpectation::Exact,
        )
    }

    pub fn require_abs_similar_nonlf(&mut self, kind: TokenKind) -> Option<TokenGraph> {
        self.require_nonlf(
            kind,
            TokenConsumptionKind::Absolute,
            TokenMatchExpectation::Similar,
        )
    }

    pub fn require_terminator(&mut self) -> Option<TokenGraph> {
        self.require(
            TokenKind::LnFeed,
            TokenConsumptionKind::UponSuccess,
            vec![],
            TokenMatchExpectation::Exact,
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
        let mut program = Program::new(Vec::new());

        while !self.stream.at_eof() {
            if let Ok(item) = self.parse_item_with_recovery() {
                program.items.push(item);
            }
        }

        program
    }

    pub fn sync(&mut self, with: Vec<TokenKind>) {
        self.stream.adjustn(self.stream.first_of_offset(with) + 1);
        self.dctx.sync();
    }

    pub fn try_sync(&mut self, with: Vec<TokenKind>) {
        if self.dctx.is_in_disarray() {
            self.sync(with);
        }
    }
}
