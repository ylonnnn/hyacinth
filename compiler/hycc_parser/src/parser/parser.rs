use hycc_ast::{
    item::{Petal, PetalKind},
    token::{Token, TokenGraph, TokenKind},
    token_stream::{TokenConsumptionKind, TokenMatchExpectation, TokenStream},
};
use hycc_diagnostic::DiagnosticContext;
use hycc_source::Source;
use hycc_span::Span;

use crate::parser::diag::{
    ParserDiag, ParserDiagCtx, ParserDiagErrorKind, UnexpectedTokenExpectation,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseLevel {
    Global,
    Local,
}

#[derive(Debug)]
pub struct Parser<'s> {
    pub(super) stream: TokenStream,
    pub dctx: ParserDiagCtx,
    pub(super) source: &'s Source,

    pub(super) level: ParseLevel,
    pub(super) petal_stack: Vec<String>,
}

pub type ParseResult<T, E = Option<ParserDiag>> = Result<T, E>;

impl<'s> Parser<'s> {
    pub fn new(stream: TokenStream, source: &'s Source) -> Self {
        Self {
            stream,
            dctx: ParserDiagCtx::new(),
            source,

            level: ParseLevel::Global,
            petal_stack: Vec::new(),
        }
    }

    pub fn eos(&self) -> bool {
        if let Some(tok) = self.peek_nonlf_token() {
            tok.kind == TokenKind::Eos || tok.kind == TokenKind::Eof
        } else {
            true
        }
    }

    pub fn adjust_to_nonlf(&mut self) {
        self.stream
            .adjustn(self.stream.first_not_offset(&[TokenKind::LnFeed]) + 1);
    }

    pub fn peek_nonlf(&self) -> Option<&TokenGraph> {
        self.peekn_nonlf(0)
    }

    pub fn peekn_nonlf(&self, offset: usize) -> Option<&TokenGraph> {
        let offset = offset + self.stream.first_not_offset(&[TokenKind::LnFeed]);
        self.stream.peekn(offset)
    }

    pub fn peek_nonlf_token(&self) -> Option<&Token> {
        self.peek_nonlf()?.underlying()
    }

    pub fn next_nonlf(&mut self) -> Option<TokenGraph> {
        let offset = self.stream.first_not_offset(&[TokenKind::LnFeed]) + 1;
        let peek = self.stream.peekn(offset - 1).cloned();

        self.stream.adjustn(offset);
        let tok = peek?.clone();

        Some(tok)
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
            .expect(kind, consumption, &[TokenKind::LnFeed], expectation)
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
        exclude: &[TokenKind],
        expectation: TokenMatchExpectation,
    ) -> Result<TokenGraph, Option<ParserDiag>> {
        let (matched, tg) = self.stream.expect(kind, consumption, exclude, expectation);
        let Some(tg) = tg else { return Err(None) };

        if !matched {
            match tg.underlying() {
                None => Err(None),
                Some(tok) => Err(Some(ParserDiag::error(
                    tok.span,
                    ParserDiagErrorKind::UnexpectedToken {
                        token: tok.clone(),
                        expected: Some(UnexpectedTokenExpectation::TokenKind(kind)),
                    },
                ))),
            }
        } else {
            Ok(tg)
        }
    }

    pub fn require_nonlf(
        &mut self,
        kind: TokenKind,
        consumption: TokenConsumptionKind,
        expectation: TokenMatchExpectation,
    ) -> Result<TokenGraph, Option<ParserDiag>> {
        self.require(kind, consumption, &[TokenKind::LnFeed], expectation)
    }

    pub fn require_exact_nonlf(
        &mut self,
        kind: TokenKind,
    ) -> Result<TokenGraph, Option<ParserDiag>> {
        self.require_nonlf(
            kind,
            TokenConsumptionKind::UponSuccess,
            TokenMatchExpectation::Exact,
        )
    }

    pub fn require_similar_nonlf(
        &mut self,
        kind: TokenKind,
    ) -> Result<TokenGraph, Option<ParserDiag>> {
        self.require_nonlf(
            kind,
            TokenConsumptionKind::UponSuccess,
            TokenMatchExpectation::Similar,
        )
    }

    pub fn require_abs_exact_nonlf(
        &mut self,
        kind: TokenKind,
    ) -> Result<TokenGraph, Option<ParserDiag>> {
        self.require_nonlf(
            kind,
            TokenConsumptionKind::Absolute,
            TokenMatchExpectation::Exact,
        )
    }

    pub fn require_abs_similar_nonlf(
        &mut self,
        kind: TokenKind,
    ) -> Result<TokenGraph, Option<ParserDiag>> {
        self.require_nonlf(
            kind,
            TokenConsumptionKind::Absolute,
            TokenMatchExpectation::Similar,
        )
    }

    pub fn require_terminator(&mut self) -> Result<TokenGraph, Option<ParserDiag>> {
        self.require(
            TokenKind::LnFeed,
            TokenConsumptionKind::UponSuccess,
            &[],
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

    pub fn parse(&mut self) -> Petal {
        let mut petal = Petal::new(
            PetalKind::Root,
            Vec::new(),
            Span::dummy(self.source.identifier.0),
        );

        while !self.stream.at_eof() {
            match self.parse_item_with_recovery() {
                Ok(item) => petal.items.push(item),
                Err(err) => {
                    if let Some(err) = err {
                        self.dctx.add(err);
                    }
                }
            }
        }

        petal
    }

    pub fn sync(&mut self, with: &[TokenKind]) {
        self.stream.adjustn(self.stream.first_of_offset(with) + 1);
        self.dctx.sync();
    }

    pub fn try_sync(&mut self, with: &[TokenKind]) {
        if self.dctx.is_in_disarray() {
            self.sync(with);
        }
    }
}
