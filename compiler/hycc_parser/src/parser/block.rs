use hycc_ast::{
    Block,
    token::{TokenGraph, TokenKind},
    token_stream::TokenStream,
};

use crate::parser::Parser;

impl<'d, 's> Parser<'d, 's> {
    pub fn parse_block(&mut self) -> Option<Block> {
        let TokenGraph::Collection { data, .. } = self.require_exact_nonlf(TokenKind::LeftBrace)?
        else {
            return None;
        };

        self.use_stream(TokenStream::new(data), |s| -> Option<Block> {
            let span = s.stream.next()?.underlying()?.span;

            let mut stmts = Vec::new();
            while !s.stream.at_eof() {
                if let Some(stmt) = s.parse_stmt() {
                    stmts.push(stmt)
                }
            }

            Some(Block {
                stmts,
                span: s.stream.next()?.underlying()?.span.merge(&span),
            })
        })
    }
}
