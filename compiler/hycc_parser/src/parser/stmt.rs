use hycc_ast::{
    Stmt, StmtKind,
    stmt::{PassStmt, RetStmt},
    token::{TokenIdentKind, TokenKind},
};

use crate::parser::{
    Parser,
    diag::ParserDiag,
    parser::{ParseResult, ParserTerminatorKind},
};

impl<'s> Parser<'s> {
    pub fn parse_stmt_with_recovery(&mut self) -> ParseResult<Stmt> {
        let stmt = self.parse_stmt();
        self.try_sync(&[TokenKind::LnFeed, TokenKind::RightBrace]);

        stmt
    }

    pub fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        let Some(tok) = self.peek_nonlf_token() else {
            return Err(None);
        };

        let tok = tok.clone();
        let stmt = self.try_parse_stmt();

        match stmt {
            Err(None) => Err(Some(ParserDiag::unexpected_token_expected_arbitrary(
                tok,
                "a statement",
            ))),
            _ => stmt,
        }
    }

    pub fn try_parse_stmt(&mut self) -> ParseResult<Stmt> {
        self.stream.save_offset();
        let Some(tok) = self.peek_nonlf_token().cloned() else {
            return Err(None);
        };

        match tok.kind {
            // TODO: implement other statements
            TokenKind::Ident(TokenIdentKind::Ret) => {
                self.adjust_to_nonlf();

                let value = if self.eos() {
                    None
                } else {
                    match self.parse_expr(0) {
                        Ok(expr) => Some(Box::new(expr)),
                        Err(None) => None,
                        Err(err) => return Err(err),
                    }
                };

                self.require_terminator(ParserTerminatorKind::Both)?;

                Ok(Stmt::new(StmtKind::Ret(Box::new(RetStmt {
                    span: value
                        .as_ref()
                        .map_or(tok.span, |val| val.span.merge(tok.span)),
                    value,
                }))))
            }

            TokenKind::Ident(TokenIdentKind::Pass) => {
                self.adjust_to_nonlf();

                let value = if self.eos() {
                    None
                } else {
                    match self.parse_expr(0) {
                        Ok(expr) => Some(Box::new(expr)),
                        Err(None) => None,
                        Err(err) => return Err(err),
                    }
                };

                Ok(Stmt::new(StmtKind::Pass(Box::new(PassStmt {
                    span: value
                        .as_ref()
                        .map_or(tok.span, |val| val.span.merge(tok.span)),
                    label: None, // TODO: block label used in pass
                    value,
                }))))
            }

            _ => {
                self.stream.revert();
                self.stream.save_offset();

                match self.try_parse_item_with_recovery() {
                    Ok(item) => Ok(Stmt::new(StmtKind::Item(Box::new(item)))),
                    Err(err) => {
                        if err.is_some() {
                            Err(err)
                        } else {
                            self.stream.revert();
                            match self.parse_expr_stmt() {
                                Ok(expr) => Ok(Stmt::new(StmtKind::Expr(Box::new(expr)))),
                                Err(err) => {
                                    self.sync(&[TokenKind::LnFeed, TokenKind::RightBrace]);
                                    Err(err)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
