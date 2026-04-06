use hycc_ast::{
    Program,
    token::{Token, TokenGraph, TokenKind},
    token_stream::{TokenConsumptionKind, TokenMatchExpectation, TokenStream},
};
use hycc_diagnostic::DiagnosticContext;
use hycc_source::Source;

use crate::parser::diag::{
    ParserDiag, ParserDiagCtx, ParserDiagErrorKind, UnexpectedTokenExpectation,
};

#[derive(Debug)]
pub struct Parser<'s> {
    pub(super) stream: TokenStream,
    pub dctx: ParserDiagCtx,
    pub(super) source: &'s Source,

    pub(super) generic_delimeter_encounters: usize,
}

pub type ParseResult<T, E = Option<ParserDiag>> = Result<T, E>;

impl<'s> Parser<'s> {
    pub fn new(source: &'s Source, stream: TokenStream) -> Self {
        Self {
            stream,
            dctx: ParserDiagCtx::new(),
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
        self.require(kind, consumption, vec![TokenKind::LnFeed], expectation)
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
            match self.parse_item_with_recovery() {
                Ok(item) => program.items.push(item),
                Err(err) => {
                    if let Some(err) = err {
                        self.dctx.add(err);
                    }
                }
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
