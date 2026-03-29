use hycc_ast::{
    Ty, TyKind,
    token::{TokenGraph, TokenIdentKind, TokenKind},
    token_stream::TokenStream,
};
use hycc_diagnostic::DiagnosticContext;

use crate::{
    errors,
    parser::{Parser, parser::ParseResult},
};

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

impl<'d, 's> Parser<'d, 's> {
    pub fn parse_ty(&mut self) -> ParseResult<Ty> {
        let Some(tok) = self.peek_nonlf_token() else {
            return Err(false);
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
                self.dctx.add(errors::unexpected_token(
                    self.source,
                    &tok,
                    Some("expected type"),
                ));

                Err(false)
            }
        }?;

        let Some(tok) = self.peek_nonlf_token() else {
            return Ok(ty);
        };

        match tok.kind {
            TokenKind::Eq | TokenKind::Comma | TokenKind::LeftBrace => return Ok(ty),

            _ => {
                self.dctx.add(errors::unexpected_token(
                    self.source,
                    &tok,
                    Some("expected type suffix"),
                ));

                Err(true)
            }
        }
    }

    pub fn parse_grouped_ty(&mut self) -> ParseResult<Ty> {
        let Some(TokenGraph::Collection { data, .. }) =
            self.require_abs_exact_nonlf(TokenKind::LeftParen)
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
            |s| {
                if s.stream.is_empty() {
                    return Ok(Ty::new(TyKind::Unit(span)));
                }

                let inner = s.parse_ty();
                if !s.stream.abs_eof() {
                    let Some(tok) = s.peek_nonlf_token() else {
                        return Err(false);
                    };

                    s.dctx.add(errors::unexpected_token(self.source, tok, None));
                }

                inner
            },
        )
    }

    // PATH
    pub fn parse_path_ty(&mut self) -> ParseResult<Ty> {
        Ok(Ty::new(TyKind::Path(Box::new(self.parse_path()?))))
    }

    // pub fn parse_ty(&mut self, min_bp: u8) -> ParseResult<Ty> {
    //     let Ok(mut prefix) = self.parse_prefix_ty() else {
    //         return Err(false);
    //     };

    //     while !self.stream.abs_eof() {
    //         let Some(tok) = self.peek_nonlf_token() else {
    //             return Err(true);
    //         };

    //         let rbp = match Self::ty_infix_binding_power_of(tok.kind) {
    //             Some((lbp, rbp)) => ternary!(min_bp > lbp, break, rbp),
    //             _ => {
    //                 break;
    //             }
    //         };

    //         let Ok(infix) = self.parse_infix_ty(&prefix, rbp) else {
    //             break;
    //         };

    //         prefix = infix;
    //     }

    //     Ok(prefix)
    // }

    // pub fn parse_prefix_ty(&mut self) -> ParseResult<Ty> {
    //     let Some(token) = self.peek_nonlf_token() else {
    //         return Err(false);
    //     };

    //     match token.kind {
    //         TokenKind::LeftParen => {
    //         }

    //         TokenKind::Ident(..) => Ok(Ty::new(TyKind::Path(Box::new(self.parse_path()?)))),

    //         _ => {
    //             self.dctx.add(errors::unexpected_token(
    //                 self.source,
    //                 &token,
    //                 Some("expected type prefix token"),
    //             ));

    //             Err(false)
    //         }
    //     }
    // }

    // pub fn parse_infix_ty(&mut self, _left: &Ty, _min_bp: u8) -> ParseResult<Ty> {
    //     let Some(token) = self.peek_nonlf_token() else {
    //         return Err(false);
    //     };

    //     match token.kind {
    //         _ => {
    //             self.dctx.add(errors::unexpected_token(
    //                 self.source,
    //                 &token,
    //                 Some("expected type infix token"),
    //             ));

    //             Err(true)
    //         }
    //     }
    // }
}
