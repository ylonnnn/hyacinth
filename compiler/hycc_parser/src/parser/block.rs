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

        let n = data.len();
        let span = data
            .first()
            .unwrap()
            .underlying()
            .unwrap()
            .span
            .merge(&data.last().unwrap().underlying().unwrap().span);

        self.use_stream(
            TokenStream::new(data.into_iter().skip(1).take(n - 2).collect()),
            |s| -> ParseResult<Block> {
                let mut stmts = Vec::new();
                while !s.stream.at_eof() {
                    if let Ok(stmt) = s.parse_stmt_with_recovery() {
                        stmts.push(stmt)
                    }
                }

                Ok(Block { stmts, span })
            },
        )
    }
}
