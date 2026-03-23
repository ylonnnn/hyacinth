use hycc_ast::{
    Program,
    token::{TokenGraph, TokenKind},
    token_stream::{TokenConsumptionKind, TokenStream},
};
use hycc_diagnostic::DiagnosticContext;
use hycc_source::Source;

use crate::{errors, parser::diag_ctx::ParserDiagCtx};

#[derive(Debug)]
pub struct Parser<'d, 's> {
    pub(super) stream: TokenStream,
    dctx: ParserDiagCtx<'d>,
    source: &'s Source,
}

impl<'d, 's> Parser<'d, 's> {
    pub fn new(source: &'s Source, dctx: ParserDiagCtx<'d>, stream: TokenStream) -> Self {
        Self {
            stream,
            dctx,
            source,
        }
    }

    pub fn expect(
        &mut self,
        kind: TokenKind,
        consumption: TokenConsumptionKind,
        exclude: Vec<TokenKind>,
    ) -> (bool, Option<TokenGraph>) {
        self.stream.expect(kind, consumption, exclude)
    }

    pub fn require(
        &mut self,
        kind: TokenKind,
        consumption: TokenConsumptionKind,
        exclude: Vec<TokenKind>,
    ) -> Option<TokenGraph> {
        let (matched, tg) = self.expect(kind, consumption, exclude);
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

    pub fn parse(&mut self) -> Program {
        let program = Program::new(Vec::new());

        while !self.stream.at_eof() {
            self.parse_item();
        }

        program
    }
}
