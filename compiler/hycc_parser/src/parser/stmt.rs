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
        let Some(tok) = self.next_nonlf_token() else {
            return Err(false);
        };

        match tok.kind {
            // TODO: implement other statements
            // TokenKind::Ident(..) => None,
            _ => {
                self.stream.save_offset();

                if let Ok(item) = self.try_parse_item() {
                    Ok(Stmt::new(StmtKind::Item(item)))
                } else if let Ok(expr) = self.parse_expr(0) {
                    self.stream.revert();
                    Ok(Stmt::new(StmtKind::Expr(expr)))
                } else {
                    self.stream.revert();
                    Err(false)
                }
            }
        }
    }
}
