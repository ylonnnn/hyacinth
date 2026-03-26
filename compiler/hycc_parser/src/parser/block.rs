use hycc_ast::{
    Block,
    token::{TokenGraph, TokenKind},
    token_stream::TokenStream,
};

use crate::parser::{Parser, parser::ParseResult};

impl<'d, 's> Parser<'d, 's> {
    pub fn parse_block(&mut self) -> ParseResult<Block> {
        let Some(TokenGraph::Collection { data, .. }) =
            self.require_exact_nonlf(TokenKind::LeftBrace)
        else {
            return Err(true);
        };

        self.use_stream(TokenStream::new(data), |s| -> ParseResult<Block> {
            let Some(tok) = s.next_nonlf_token() else {
                return Err(false);
            };

            let span = tok.span;
            let mut stmts = Vec::new();

            while !s.stream.at_eof() {
                if let Ok(stmt) = s.parse_stmt_with_recovery() {
                    stmts.push(stmt)
                }
            }

            let Some(tok) = s.next_nonlf_token() else {
                return Err(false);
            };

            Ok(Block {
                stmts,
                span: tok.span.merge(&span),
            })
        })
    }
}
