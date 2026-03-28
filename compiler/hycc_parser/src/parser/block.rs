use hycc_ast::{
    Block,
    token::{TokenGraph, TokenKind},
    token_stream::TokenStream,
};
use hycc_util::ternary;

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
                let mut data = Block {
                    stmts: Vec::new(),
                    span,
                };

                if s.stream.is_empty() {
                    return Ok(data);
                }

                while !s.stream.at_eof() {
                    match s.parse_stmt_with_recovery() {
                        Ok(stmt) => data.stmts.push(stmt),
                        Err(matched) => ternary!(matched, continue, break),
                    }
                }

                Ok(data)
            },
        )
    }
}
