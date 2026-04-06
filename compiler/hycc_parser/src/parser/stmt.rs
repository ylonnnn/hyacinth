use hycc_ast::{Stmt, StmtKind, token::TokenKind};

use crate::parser::{
    Parser,
    diag::{ParserDiag, ParserDiagErrorKind, UnexpectedTokenExpectation},
    parser::ParseResult,
};

impl<'s> Parser<'s> {
    pub fn parse_stmt_with_recovery(&mut self) -> ParseResult<Stmt> {
        let stmt = self.parse_stmt();
        self.try_sync(vec![TokenKind::LnFeed, TokenKind::RightBrace]);

        stmt
    }

    pub fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        let Some(tok) = self.peek_nonlf_token() else {
            return Err(None);
        };

        let tok = tok.clone();
        let stmt = self.try_parse_stmt();

        match stmt {
            Err(None) => Err(Some(ParserDiag::error(
                tok.span,
                ParserDiagErrorKind::UnexpectedToken {
                    token: tok,
                    expected: Some(UnexpectedTokenExpectation::Arbitrary("an item")),
                },
            ))),
            _ => stmt,
        }
    }

    pub fn try_parse_stmt(&mut self) -> ParseResult<Stmt> {
        self.stream.save_offset();
        let Some(tok) = self.next_nonlf_token() else {
            return Err(None);
        };

        match tok.kind {
            // TODO: implement other statements
            // TokenKind::Ident(..) => None,
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
                            match self.try_parse_expr_stmt() {
                                Ok(expr) => Ok(Stmt::new(StmtKind::Expr(Box::new(expr)))),
                                Err(err) => {
                                    if err.is_some() {
                                        self.sync(vec![TokenKind::LnFeed, TokenKind::RightBrace]);
                                    }

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
