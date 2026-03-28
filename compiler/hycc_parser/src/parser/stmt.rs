use hycc_ast::{Stmt, StmtKind, token::TokenKind};
use hycc_diagnostic::DiagnosticContext;

use crate::{
    errors,
    parser::{Parser, parser::ParseResult},
};

impl<'d, 's> Parser<'d, 's> {
    pub fn parse_stmt_with_recovery(&mut self) -> ParseResult<Stmt> {
        let stmt = self.parse_stmt();
        self.try_sync(vec![TokenKind::LnFeed, TokenKind::RightBrace]);

        stmt
    }

    pub fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        let Some(tok) = self.peek_nonlf_token() else {
            return Err(true);
        };

        let tok = tok.clone();
        let stmt = self.try_parse_stmt();

        if let Err(matched) = stmt
            && !matched
        {
            self.dctx.add(errors::unexpected_token(
                self.source,
                &tok,
                Some("expected a statement"),
            ));
        }

        stmt
    }

    pub fn try_parse_stmt(&mut self) -> ParseResult<Stmt> {
        self.stream.save_offset();
        let Some(tok) = self.next_nonlf_token() else {
            return Err(false);
        };

        match tok.kind {
            // TODO: implement other statements
            // TokenKind::Ident(..) => None,
            _ => {
                self.stream.revert();
                self.stream.save_offset();

                match self.try_parse_item_with_recovery() {
                    Ok(item) => return Ok(Stmt::new(StmtKind::Item(Box::new(item)))),
                    Err(true) => Err(true)?,
                    Err(false) => {
                        self.stream.revert();
                        match self.try_parse_expr_stmt() {
                            Ok(expr) => Ok(Stmt::new(StmtKind::Expr(Box::new(expr)))),

                            Err(true) => {
                                self.sync(vec![TokenKind::LnFeed, TokenKind::RightBrace]);
                                Err(true)
                            }

                            Err(false) => Err(false),
                        }
                    }
                }
            }
        }
    }
}
