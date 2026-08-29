use hycc_ast::{
    item::{Petal, PetalKind},
    token::{Token, TokenGraph, TokenKind},
    token_stream::{TokenConsumptionKind, TokenMatchExpectation, TokenStream},
};
use hycc_diagnostic::diagnostic::{DiagCtx, Diagnostics};
use hycc_source::Source;
use hycc_span::Span;
use hycc_util::ternary;

use crate::parser::diag::{ParserDiag, ParserDiagCtx};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserCtx {
    Normal,

    // Disables struct literal parsing for if conditions unless
    // surrounded by `()` to avoid ambiguity.
    IfCond,
}

#[derive(Debug)]
pub struct Parser<'p> {
    pub(super) stream: TokenStream,
    pub dctx: ParserDiagCtx<'p>,
    pub(super) petal_stack: Vec<String>,
    pub(super) source: &'p Source,
    pub(super) depth: usize,
    pub(super) ctx: ParserCtx,
}

#[derive(Debug, Clone, Copy)]
pub enum ParserTerminatorKind {
    LnFeed,
    SemiColon,
    Both,
}

impl<'p> Parser<'p> {
    pub fn new(stream: TokenStream, source: &'p Source, dctx: &'p mut DiagCtx) -> Self {
        Self {
            stream,
            dctx: ParserDiagCtx::new(dctx),
            petal_stack: Vec::new(),
            source,
            depth: 0,
            ctx: ParserCtx::Normal,
        }
    }

    pub fn use_ctx<F, T>(&mut self, ctx: ParserCtx, mut handler: F) -> T
    where
        F: FnMut(&mut Self) -> T,
    {
        let prev_ctx = self.ctx;
        self.ctx = ctx;
        let data = handler(self);
        self.ctx = prev_ctx;

        data
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
        let (matched, tokg) = self.stream.expect(kind, consumption, exclude, expectation);
        let Some(tokg) = tokg else { return Err(None) };

        if !matched {
            match tokg.underlying() {
                None => Err(None),
                Some(tok) => Err(Some(ParserDiag::unexpected_token_expected_token(
                    tok.clone(),
                    kind,
                ))),
            }
        } else {
            Ok(tokg)
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

    pub fn require_terminator(
        &mut self,
        kind: ParserTerminatorKind,
    ) -> Result<TokenGraph, Option<ParserDiag>> {
        let expected = match &kind {
            ParserTerminatorKind::LnFeed => TokenKind::LnFeed.to_string(),
            ParserTerminatorKind::SemiColon => TokenKind::SemiColon.to_string(),
            ParserTerminatorKind::Both => "terminator".into(),
        };

        let res = match &kind {
            ParserTerminatorKind::LnFeed => self.stream.expect(
                TokenKind::LnFeed,
                TokenConsumptionKind::UponSuccess,
                &[],
                TokenMatchExpectation::Exact,
            ),

            ParserTerminatorKind::SemiColon => self.expect_exact_nonlf(TokenKind::SemiColon),

            ParserTerminatorKind::Both => {
                let lf_term = self.stream.expect(
                    TokenKind::LnFeed,
                    TokenConsumptionKind::UponSuccess,
                    &[],
                    TokenMatchExpectation::Exact,
                );

                ternary!(
                    lf_term.0,
                    lf_term,
                    self.expect_exact_nonlf(TokenKind::SemiColon)
                )
            }
        };

        ternary!(
            res.0,
            Ok(res.1.unwrap()),
            self.expect_preserved_exact_nonlf(TokenKind::RightBrace)
                .1
                .ok_or_else(|| {
                    let token = self.peek_nonlf_token()?;
                    Some(ParserDiag::unexpected_token_expected_arbitrary(
                        token.clone(),
                        "terminator",
                    ))
                })
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

        while !self.eos() {
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
