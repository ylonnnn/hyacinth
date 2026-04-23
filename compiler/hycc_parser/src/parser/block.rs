use hycc_ast::{
    Block,
    token::{TokenGraph, TokenKind},
    token_stream::TokenStream,
};
use hycc_diagnostic::DiagnosticContext;

use crate::parser::{
    Parser,
    parser::{ParseLevel, ParseResult},
};

impl<'s> Parser<'s> {
    pub fn parse_block(&mut self) -> ParseResult<Block> {
        let data = match self.require_exact_nonlf(TokenKind::LeftBrace)? {
            TokenGraph::Collection { data, .. } => data,
            _ => Err(None)?,
        };

        let n = data.len();
        let span = data
            .first()
            .unwrap()
            .underlying()
            .unwrap()
            .span
            .merge(&data.last().unwrap().underlying().unwrap().span);

        let prev_level = self.level;
        self.level = ParseLevel::Local;

        let block = self.use_stream(
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
                        Err(diag) => {
                            if let Some(diag) = diag {
                                s.dctx.add(diag);
                            }
                        }
                    }
                }

                Ok(data)
            },
        );

        self.level = prev_level;
        block
    }
}
