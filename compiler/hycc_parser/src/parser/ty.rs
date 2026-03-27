use hycc_ast::{
    Ty, TyKind,
    token::{TokenGraph, TokenIdentKind, TokenKind},
    token_stream::TokenStream,
};
use hycc_diagnostic::DiagnosticContext;
use hycc_util::ternary;

use crate::{
    errors,
    parser::{Parser, parser::ParseResult},
};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum TyInfixBindingPower {
    Default,
    Fn,
    Ref,
    Ptr,
    Array,
    Primary,
}

impl<'d, 's> Parser<'d, 's> {
    pub fn ty_infix_binding_power_of(kind: TokenKind) -> Option<(u8, u8)> {
        use TyInfixBindingPower::*;

        match kind {
            TokenKind::LeftParen => Some((Default as u8, Default as u8)),

            TokenKind::Ampersand => Some((Ref as u8, Ref as u8)),
            TokenKind::Star => Some((Ptr as u8, Ptr as u8)),
            TokenKind::LeftBracket => Some((Array as u8, Array as u8)),

            TokenKind::Ident(kind) => match kind {
                TokenIdentKind::Normal => Some((Primary as u8, Primary as u8)),
                TokenIdentKind::Fn => Some((Fn as u8, Fn as u8)),

                _ => None,
            },

            _ => None,
        }
    }

    pub fn parse_ty(&mut self, min_bp: u8) -> ParseResult<Ty> {
        let Ok(mut prefix) = self.parse_prefix_ty() else {
            return Err(false);
        };

        while !self.stream.abs_eof() {
            let Some(tok) = self.peek_nonlf_token() else {
                return Err(true);
            };

            let rbp = match Self::ty_infix_binding_power_of(tok.kind) {
                Some((lbp, rbp)) => ternary!(min_bp > lbp, break, rbp),
                _ => {
                    break;
                }
            };

            let Ok(infix) = self.parse_infix_ty(&prefix, rbp) else {
                break;
            };

            prefix = infix;
        }

        Ok(prefix)
    }

    pub fn parse_prefix_ty(&mut self) -> ParseResult<Ty> {
        let Some(token) = self.peek_nonlf_token() else {
            return Err(false);
        };

        match token.kind {
            TokenKind::LeftParen => {
                // TODO: allow the parser to diverge from a grouped type, or a tuple
                self.parse_grouped_ty()
            }

            TokenKind::Ident(..) => Ok(Ty::new(TyKind::Path(Box::new(self.parse_path()?)))),

            _ => {
                self.dctx.add(errors::unexpected_token(
                    self.source,
                    &token,
                    Some("expected type prefix token"),
                ));

                Err(false)
            }
        }
    }

    pub fn parse_infix_ty(&mut self, _left: &Ty, _min_bp: u8) -> ParseResult<Ty> {
        let Some(token) = self.peek_nonlf_token() else {
            return Err(false);
        };

        match token.kind {
            _ => {
                self.dctx.add(errors::unexpected_token(
                    self.source,
                    &token,
                    Some("expected type infix token"),
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

                let inner = s.parse_ty(0);
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
}
