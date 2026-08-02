use hycc_ast::{
    Mutability, Ty, TyKind,
    token::{TokenGraph, TokenIdentKind, TokenKind},
    token_stream::TokenStream,
    ty::{Array, FnTy, Ref, Slice, Tuple, TyParam, TyParamList},
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
        let tg = self.require_abs_exact_nonlf(TokenKind::LeftParen)?;
        let span = tg.span();

        let TokenGraph::Collection { data, .. } = tg else {
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
                    let Some(tg) = s.peek_nonlf() else {
                        unreachable!()
                    };

                    return Err(Some(ParserDiag::unexpected_token(
                        tg.underlying().unwrap().clone(),
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

        let tg = self.require_abs_exact_nonlf(TokenKind::LeftParen)?;
        let params_span = tg.span();
        let TokenGraph::Collection { data, .. } = tg else {
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

    // TYPE_PARAM (, TYPE_PARAM) >
    pub fn parse_ty_params(&mut self) -> ParseResult<TyParamList> {
        let Some(op_delim) = self.next_nonlf_token() else {
            return Err(None);
        };

        let mut param_decls = TyParamList {
            list: Vec::new(),
            span: op_delim.span,
        };

        let mut expect = true;
        while !dbg!(self.expect_exact_nonlf(TokenKind::Greater)).0 {
            if !expect {
                self.require_exact_nonlf(TokenKind::SemiColon)?;
            }

            if expect {
                param_decls.list.push(self.parse_ty_param()?);
                expect = false;
            }

            if !expect && dbg!(self.expect_exact_nonlf(TokenKind::SemiColon)).0 {
                expect = true;
                continue;
            }
        }

        Ok(param_decls)
    }

    // RAW_IDENT (: PROTO_REQS)?
    // RAW_IDENT (: PROTO (, PROTO)* )
    pub fn parse_ty_param(&mut self) -> ParseResult<TyParam> {
        // RAW_IDENT
        let param = self.parse_raw_ident()?;

        // :
        let mut proto_reqs = Vec::new();
        if self.expect_exact_nonlf(TokenKind::Colon).0 {
            let mut expect = true;
            while !self.expect_preserved_exact_nonlf(TokenKind::SemiColon).0
                && !self.expect_preserved_exact_nonlf(TokenKind::Greater).0
            {
                if !expect {
                    self.require_exact_nonlf(TokenKind::Comma)?;
                }

                if expect {
                    // TODO: use (and make) parse_proto_ident or allow associated
                    // types to be parsed in the identifier arguments
                    proto_reqs.push(self.parse_ident(PathKind::Ty)?);
                    expect = false;
                }

                if !expect && self.expect_exact_nonlf(TokenKind::Comma).0 {
                    expect = true;
                    continue;
                }
            }
        }

        Ok(TyParam {
            ident: param,
            proto_reqs,
        })
    }
}
