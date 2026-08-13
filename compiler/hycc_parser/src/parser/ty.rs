use hycc_ast::{
    Mutability,
    token::{TokenGraph, TokenIdentKind, TokenKind},
    token_stream::TokenStream,
    ty::{Array, FnTy, Ref, Slice, Tuple},
    ty::{Ty, TyKind},
};
use hycc_diagnostic::DiagnosticContext;
use hycc_util::ternary;

use crate::parser::{Parser, diag::ParserDiag, parser::ParseResult, path::PathKind};

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
            TokenKind::LeftParen => Ok(Ty::new(self.parse_paren_enclosed_ty()?)),

            TokenKind::Ampersand => Ok(Ty::new(TyKind::Ref(Box::new(self.parse_ref_ty()?)))),

            TokenKind::LeftBracket => Ok(Ty::new(self.parse_array_or_slice_ty()?)),

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
                        Ok(Ty::new(TyKind::Fn(Box::new(self.parse_fn_ty()?))))
                    }
                    _ => self.parse_path_ty(),
                }
            }

            // TODO: improve
            TokenKind::Eq | TokenKind::Comma => {
                // return Err(None);
                todo!(
                    "perhaps you forgot the type, or did not mean to explicitly add the type annotation"
                );
            }

            _ => Err(Some(ParserDiag::unexpected_token_expected_arbitrary(
                tok.clone(),
                "type",
            ))),
        }?;

        // let Some(tok) = self.peek_nonlf_token() else {
        //     return Ok(ty);
        // };

        // match tok.kind {
        //     TokenKind::Eq
        //     | TokenKind::Comma
        //     | TokenKind::LeftBrace
        //     | TokenKind::RightParen
        //     | TokenKind::Greater
        //     | TokenKind::GreaterGreater => {
        //         return Ok(ty);
        //     }

        //     _ => {
        //         // self.dctx.add(errors::unexpected_token(
        //         //     self.source,
        //         //     &tok,
        //         //     Some("expected type suffix"),
        //         // ));

        //         // Err(Some(ParserDiag::unexpected_token_expected_arbitrary(
        //         //     tok.clone(),
        //         //     "type suffix",
        //         // )))
        //     }
        // }

        Ok(ty)
    }

    // Parenthesis-enclosed types
    // e.g. `()`, `(&i32, bool)`
    pub fn parse_paren_enclosed_ty(&mut self) -> ParseResult<TyKind> {
        let tokg = self.require_abs_exact_nonlf(TokenKind::LeftParen)?;
        let span = tokg.span();

        let TokenGraph::Collection { data, .. } = tokg else {
            unreachable!()
        };

        let n = data.len();

        self.use_stream(
            TokenStream::new(data.into_iter().skip(1).take(n - 2).collect()),
            |s| -> ParseResult<TyKind> {
                let mut tup = Tuple {
                    data: Vec::new(),
                    span,
                };
                let mut expect = true;

                while !s.eos() {
                    if !expect {
                        s.require_exact_nonlf(TokenKind::Comma)?;
                    }

                    if expect {
                        tup.data.push(s.parse_ty()?);
                        expect = false;
                    }

                    if !expect && s.expect_exact_nonlf(TokenKind::Comma).0 {
                        expect = true;
                        continue;
                    }
                }

                Ok(if tup.data.is_empty() {
                    TyKind::Unit(span)
                } else {
                    TyKind::Tuple(Box::new(tup))
                })
            },
        )
    }

    // e.g. `&i32`, `&mut T`, `&mut Test<T>`
    pub fn parse_ref_ty(&mut self) -> ParseResult<Ref> {
        let span = self.next_nonlf_token().unwrap().span;
        let mutability = if self
            .expect_exact_nonlf(TokenKind::Ident(TokenIdentKind::Mut))
            .0
        {
            Mutability::Mutable
        } else {
            Mutability::Immutable
        };

        let ty = Box::new(self.parse_ty()?);

        Ok(Ref {
            span: span.merge(ty.span),
            ty,
            mutability,
        })
    }

    // e.g. `[]T`, `[][5]i32`, `[8]&u8`
    pub fn parse_array_or_slice_ty(&mut self) -> ParseResult<TyKind> {
        let Some(TokenGraph::Collection { data, .. }) = self.next_nonlf() else {
            unreachable!()
        };

        let n = data.len();
        let op = data.first().unwrap().underlying().unwrap().clone();

        let size = self.use_stream(
            TokenStream::new(data.into_iter().skip(1).take(n - 2).collect()),
            |s| {
                if s.eos() {
                    return Ok(None);
                }

                let expr = s.parse_expr(0);
                if !s.eos() {
                    let Some(tokg) = s.peek_nonlf() else {
                        unreachable!()
                    };

                    return Err(Some(ParserDiag::unexpected_token(
                        tokg.underlying().unwrap().clone(),
                    )));
                }

                Ok(match expr {
                    Ok(expr) => Some(expr),
                    Err(diag) => {
                        if let Some(diag) = diag {
                            s.dctx.add(diag);
                        }

                        None
                    }
                })
            },
        )?;

        let ty = Box::new(self.parse_ty()?);
        let span = op.span.merge(ty.span);

        Ok(if let Some(size) = size {
            TyKind::Array(Box::new(Array {
                size: Box::new(size),
                ty,
                span,
            }))
        } else {
            TyKind::Slice(Box::new(Slice { ty, span }))
        })
    }

    // e.g. `fn()`, `fn(i32, i32) -> i32`
    pub fn parse_fn_ty(&mut self) -> ParseResult<FnTy> {
        // fn
        let Some(lead) = self.next_nonlf_token() else {
            unreachable!()
        };

        let tokg = self.require_abs_exact_nonlf(TokenKind::LeftParen)?;
        let params_span = tokg.span();
        let TokenGraph::Collection { data, .. } = tokg else {
            unreachable!()
        };

        let n = data.len();
        let params = self.use_stream(
            TokenStream::new(data.into_iter().skip(1).take(n - 2).collect()),
            |s| -> ParseResult<Vec<Ty>> {
                let mut params = Vec::new();
                let mut expect = true;

                while !s.eos() {
                    if !expect {
                        s.require_exact_nonlf(TokenKind::Comma)?;
                    }

                    if expect {
                        params.push(s.parse_ty()?);
                        expect = false;
                    }

                    if !expect && s.expect_exact_nonlf(TokenKind::Comma).0 {
                        expect = true;
                        continue;
                    }
                }

                Ok(params)
            },
        )?;

        // -> TY
        let ret_ty = ternary!(
            self.expect_exact_nonlf(TokenKind::MinusGreater).0,
            Some(self.parse_ty()?),
            None
        );

        let span = lead.span.merge(ternary!(
            ret_ty.is_some(),
            ret_ty.as_ref().unwrap().span,
            params_span
        ));

        Ok(FnTy {
            params,
            ret_ty,
            span,
        })
    }

    // PATH
    // e.g. `sample::MyTy`, `sample_type::inner_petal::Type<i32>`
    pub fn parse_path_ty(&mut self) -> ParseResult<Ty> {
        Ok(Ty::new(TyKind::Path(Box::new(
            self.parse_path(PathKind::Ty)?,
        ))))
    }
}
