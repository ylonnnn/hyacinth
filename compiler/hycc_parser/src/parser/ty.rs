use hycc_ast::{
    Ty, TyKind,
    token::{TokenGraph, TokenIdentKind, TokenKind},
    token_stream::TokenStream,
};

use crate::parser::{Parser, diag::ParserDiag, parser::ParseResult};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentLeadTyKind {
    // Path types.
    // e.g. `std::vec`, `test`
    Path,

    // Function types.
    // e.g. `fn (i32) -> i32`,
    // NOTE: `fn` is not considered a function type if not followed by a `(`.
    Fn,
    // TODO: other identifier-lead types
}

impl<'s> Parser<'s> {
    pub fn parse_ty(&mut self) -> ParseResult<Ty> {
        let Some(tok) = self.peek_nonlf_token() else {
            return Err(None);
        };

        let ty = match tok.kind {
            TokenKind::LeftParen => {
                // TODO: allow the parser to diverge from a grouped type, or a tuple
                self.parse_grouped_ty()
            }

            TokenKind::Ident(kind) => {
                self.stream.save_offset();

                if self.next_nonlf_token().is_none() {
                    self.stream.revert();
                    return self.parse_path_ty();
                };

                let ident_ty_kind = match self.peek_nonlf_token() {
                    Some(tok) => match tok.kind {
                        TokenKind::LeftParen => IdentLeadTyKind::Fn,
                        _ => IdentLeadTyKind::Path,
                    },

                    _ => IdentLeadTyKind::Path,
                };

                self.stream.revert();

                match kind {
                    TokenIdentKind::Fn if ident_ty_kind == IdentLeadTyKind::Fn => {
                        todo!("parse fn type")
                    }
                    _ => self.parse_path_ty(),
                }
            }

            // TODO: improve
            TokenKind::Eq | TokenKind::Comma => {
                todo!(
                    "perhaps you forgot the type, or did not mean to explicitly add the type annotation"
                );
            }

            _ => {
                // self.dctx.add(errors::unexpected_token(
                //     self.source,
                //     &tok,
                //     Some("expected type"),
                // ));

                Err(Some(ParserDiag::unexpected_token_expected_arbitrary(
                    tok.clone(),
                    "type",
                )))
            }
        }?;

        let Some(tok) = self.peek_nonlf_token() else {
            return Ok(ty);
        };

        match tok.kind {
            TokenKind::Eq | TokenKind::Comma | TokenKind::LeftBrace | TokenKind::RightParen => {
                return Ok(ty);
            }

            _ => {
                // self.dctx.add(errors::unexpected_token(
                //     self.source,
                //     &tok,
                //     Some("expected type suffix"),
                // ));

                Err(Some(ParserDiag::unexpected_token_expected_arbitrary(
                    tok.clone(),
                    "type suffix",
                )))
            }
        }
    }

    pub fn parse_grouped_ty(&mut self) -> ParseResult<Ty> {
        let data = match self.require_abs_exact_nonlf(TokenKind::LeftParen)? {
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

        self.use_stream(
            TokenStream::new(data.into_iter().skip(1).take(n - 2).collect()),
            |s| {
                if s.stream.is_empty() {
                    return Ok(Ty::new(TyKind::Unit(span)));
                }

                let inner = s.parse_ty();
                if !s.stream.abs_eof() {
                    let Some(tok) = s.peek_nonlf_token() else {
                        return Err(None);
                    };

                    return Err(Some(ParserDiag::unexpected_token(tok.clone())));
                }

                inner
            },
        )
    }

    // PATH
    pub fn parse_path_ty(&mut self) -> ParseResult<Ty> {
        Ok(Ty::new(TyKind::Path(Box::new(self.parse_path()?))))
    }
}
