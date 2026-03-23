use hycc_diagnostic::DiagnosticContext;

use crate::lexer::token::TokenGraph;

#[derive(Debug)]
pub struct Parser<'d> {
    dctx: &'d mut DiagnosticContext,
    token_graph: TokenGraph,
}

impl<'d> Parser<'d> {
    pub fn new(dctx: &'d mut DiagnosticContext, token_graph: TokenGraph) -> Self {
        Self { dctx, token_graph }
    }
}
