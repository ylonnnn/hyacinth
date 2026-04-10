use std::path::{self, PathBuf};

use hycc_ast::{
    Expr, Item, ItemKind, Ty,
    item::{Fn, FnParam, FnParamList, ItemAccessibility, Petal, PetalKind, VarDecl},
    token::{TokenGraph, TokenIdentKind, TokenKind},
    token_stream::TokenStream,
};
use hycc_diagnostic::DiagnosticContext;
use hycc_util::ternary;

use crate::parser::{
    Parser,
    diag::{ParserDiag, ParserDiagErrorKind},
    parser::ParseResult,
};

impl<'s> Parser<'s> {
    pub fn parse_item_with_recovery(&mut self) -> ParseResult<Item> {
        let item = self.parse_item();
        if let Err(_) = item {
            self.sync(&[TokenKind::LnFeed, TokenKind::RightBrace]);
        }

        item
    }

    pub fn try_parse_item_with_recovery(&mut self) -> ParseResult<Item> {
        let item = self.try_parse_item();
        if let Err(_) = item {
            self.sync(&[TokenKind::LnFeed, TokenKind::RightBrace]);
        }

        item
    }

    pub fn parse_item(&mut self) -> ParseResult<Item> {
        let Some(tok) = self.peek_nonlf_token() else {
            return Err(None);
        };

        let tok = tok.clone();
        let item = self.try_parse_item();

        match item {
            Err(None) => Err(Some(ParserDiag::unexpected_token_expected_arbitrary(
                tok, "an item",
            ))),
            _ => item,
        }
    }

    pub fn try_parse_item(&mut self) -> ParseResult<Item> {
        let Some(tok) = self.next_nonlf_token() else {
            return Err(None);
        };

        let span = tok.span;
        let kind = match tok.kind {
            TokenKind::Ident(TokenIdentKind::Pub) => {
                return self.parse_item_with_accessibility();
            }

            TokenKind::Ident(TokenIdentKind::Petal) => {
                ItemKind::Petal(Box::new(self.parse_petal_with_recovery()?))
            }

            TokenKind::Ident(TokenIdentKind::Fn) => {
                ItemKind::Fn(Box::new(self.parse_fn_with_recovery()?))
            }

            TokenKind::Ident(TokenIdentKind::Let) => {
                ItemKind::VarDecl(Box::new(self.parse_var_decl_with_recovery()?))
            }

            _ => Err(None)?,
        };

        let mut item = Item::new(kind);
        item.span = span.merge(&item.span);

        Ok(item)
    }

    pub fn parse_item_with_accessibility(&mut self) -> ParseResult<Item> {
        let mut item = self.parse_item_with_recovery()?;
        item.accessibility = ItemAccessibility::Pub;

        Ok(item)
    }

    pub fn parse_petal_with_recovery(&mut self) -> ParseResult<Petal> {
        let data = self.parse_petal();
        self.try_sync(&[TokenKind::RightBrace]);

        data
    }

    // petal FILE
    // petal PATH { ITEM* }
    pub fn parse_petal(&mut self) -> ParseResult<Petal> {
        // PATH
        let path = self.parse_path()?;
        let is_inline =
            path.segments.len() > 1 || self.expect_preserved_exact_nonlf(TokenKind::LeftBrace).0;

        let span = path.span;
        let segment = path.segments[0].ident.clone();

        let mut petal = Petal::new(
            ternary!(
                is_inline,
                PetalKind::Inline(path),
                PetalKind::File(segment.view(&self.source.data).to_string())
            ),
            Vec::new(),
            span,
        );

        match &mut petal.kind {
            // PATH (inline petal)
            PetalKind::Inline(_) => {
                while !self.stream.at_eof() {
                    petal.items.push(self.parse_item_with_recovery()?);
                }
            }

            // PATH (file)
            // Attempt to check if the file exists and use the absolute path
            PetalKind::File(file_path) => {
                let parent_path = path::Path::new(&self.source.identifier.1).parent().unwrap();

                // TODO: fix hardcoded `.hyc` extensions and `petal.hyc` directory petal file
                let mut found = false;
                for f_petal_path in &[
                    PathBuf::from(format!("{file_path}.hyc")),
                    PathBuf::from(file_path.clone()).join("petal.hyc"),
                ] {
                    let mut path = parent_path
                        .join(&f_petal_path)
                        .to_string_lossy()
                        .to_string();

                    match std::fs::exists(&path) {
                        Ok(res) if (found = res, res).1 => {
                            std::mem::swap(file_path, &mut path);
                            break;
                        }

                        Err(err) => match err.kind() {
                            _ => panic!("an error occurred: {err:?}"),
                        },

                        _ => {}
                    }
                }

                if !found {
                    Err(Some(ParserDiag::error(
                        span,
                        ParserDiagErrorKind::UnrecognizedPetalFile {
                            name: segment.clone(),
                        },
                    )))?
                }
            }
        }

        self.require_terminator()?;

        Ok(petal)
    }

    pub fn parse_fn_with_recovery(&mut self) -> ParseResult<Fn> {
        let data = self.parse_fn();
        self.try_sync(&[TokenKind::RightBrace]);

        data
    }

    // fn IDENT(GENERIC_PARAMS)?((PARAM_LIST)?) RET_TY? BLOCK
    // fn IDENT < GENERIC_PARAM (, GENERIC_PARAM)* > ( PARAM (, PARAM)? ) RET_TY? B{ STMT* }
    pub fn parse_fn(&mut self) -> ParseResult<Fn> {
        // IDENT
        let ident = self.parse_raw_ident();

        // (PARAM (, PARAM)*)
        let params = self.parse_fn_param_list();

        // ->
        let mut ret_ty = Option::<Ty>::None;
        if self.expect_exact_nonlf(TokenKind::MinusGreater).0 {
            // RET_TY
            ret_ty = Some(self.parse_ty()?);
        }

        // { STMT* }
        let body = self.parse_block();

        self.require_terminator()?;

        Ok(Fn {
            ident: ident?,
            params: params?,
            ret_ty: ret_ty.map(Box::new),
            body: body?,
        })
    }

    // (PARAM (, PARAM)*)
    pub fn parse_fn_param_list(&mut self) -> ParseResult<FnParamList> {
        let data = match self.require_exact_nonlf(TokenKind::LeftParen) {
            Ok(TokenGraph::Collection { data, .. }) => data,
            Ok(_) => Err(None)?,
            Err(err) => Err(err)?,
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
            TokenStream::new(data.into_iter().skip(1).take(n - 1).collect()),
            |s| {
                let mut params = FnParamList {
                    span: span,
                    list: Vec::new(),
                };

                if s.stream.is_empty() || s.stream.abs_eof() {
                    return Ok(params);
                }

                match s.parse_fn_param() {
                    Ok(lead) => {
                        params.list.push(lead);
                        while s.expect_exact_nonlf(TokenKind::Comma).0 {
                            if s.stream.abs_eof() {
                                break;
                            }

                            match s.parse_fn_param() {
                                Ok(param) => params.list.push(param),
                                Err(diag) => {
                                    if let Some(diag) = diag {
                                        s.dctx.add(diag);
                                    }
                                }
                            }
                        }
                    }

                    Err(diag) => {
                        if let Some(diag) = diag {
                            s.dctx.add(diag);
                        }
                    }
                }

                if let Ok(lead) = s.parse_fn_param() {
                    params.list.push(lead);
                    while s.expect_exact_nonlf(TokenKind::Comma).0 {
                        params.list.push(s.parse_fn_param()?);
                    }
                }

                Ok(params)
            },
        )
    }

    // IDENT : TY
    pub fn parse_fn_param(&mut self) -> ParseResult<FnParam> {
        // IDENT
        let ident = self.parse_raw_ident()?;

        // :
        self.require_abs_exact_nonlf(TokenKind::Colon)?;

        // TY
        let ty = self.parse_ty()?;

        Ok(FnParam {
            ident,
            ty: Box::new(ty),
        })
    }

    pub fn parse_var_decl_with_recovery(&mut self) -> ParseResult<VarDecl> {
        let decl = self.parse_var_decl();
        self.try_sync(&[TokenKind::LnFeed]);

        decl
    }

    // TODO: allow patterns rather than just raw identifiers
    // let IDENT (: TY)? (= EXPR)? (TERM ::= '\n')
    pub fn parse_var_decl(&mut self) -> ParseResult<VarDecl> {
        // IDENT
        let ident = self.parse_raw_ident()?;

        // :
        let mut ty = Option::<Ty>::None;
        if self.expect_exact_nonlf(TokenKind::Colon).0 {
            // TY
            ty = Some(self.parse_ty()?)
        }

        // =
        let mut val = Option::<Expr>::None;
        if self.peek_nonlf().is_some() {
            // EXPR
            self.require_abs_exact_nonlf(TokenKind::Eq)?;
            val = Some(self.parse_expr(0)?);
        }

        self.require_terminator()?;

        // Validate variable declaration composition
        if ty.is_none() && val.is_none() {
            return Err(Some(ParserDiag::error(
                ident.span,
                ParserDiagErrorKind::InvalidVarDecl { ident },
            )));
        }

        Ok(VarDecl {
            ident,
            ty: ty.map(Box::new),
            val: val.map(Box::new),
        })
    }
}
