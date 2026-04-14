use std::path::{self, PathBuf};

use hycc_ast::{
    Expr, Item, ItemKind, Ty,
    item::{Fn, FnParam, FnParamList, ItemAccessibility, Petal, PetalKind, VarDecl},
    token::{TokenGraph, TokenIdentKind, TokenKind},
    token_stream::TokenStream,
};
use hycc_diagnostic::DiagnosticContext;
use hycc_session::config;

use crate::parser::{
    Parser,
    diag::{ParserDiag, ParserDiagErrorKind},
    parser::ParseResult,
    path::PathKind,
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
        let path = self.parse_path(PathKind::None)?;
        let is_inline = self.expect_preserved_exact_nonlf(TokenKind::LeftBrace).0;

        let span = path.span;
        // Default to `PetalKind::Root`
        let mut petal = Petal::new(PetalKind::Root, Vec::new(), span);

        // PATH (inline)
        if is_inline {
            let init_len = self.petal_stack.len();
            for segment in &path.segments {
                self.petal_stack
                    .push(segment.ident.view(&self.source.data).into());
            }

            petal.kind = PetalKind::Inline(path);

            let data = match self.require_exact_nonlf(TokenKind::LeftBrace)? {
                TokenGraph::Collection { data, .. } => data,
                _ => Err(None)?,
            };

            let n = data.len();
            self.use_stream(
                TokenStream::new(data.into_iter().skip(1).take(n - 2).collect()),
                |s| {
                    if s.stream.is_empty() {
                        return;
                    }

                    while !s.stream.at_eof() {
                        match s.parse_item_with_recovery() {
                            Ok(item) => petal.items.push(item),
                            Err(diag) => {
                                if let Some(diag) = diag {
                                    s.dctx.add(diag);
                                }
                            }
                        }
                    }
                },
            );

            while self.petal_stack.len() > init_len {
                self.petal_stack.pop();
            }
        }
        // PATH (file)
        // Attempt to check if the file exists and use the absolute path
        else {
            let parent_path = path::Path::new(&self.source.identifier.1).parent().unwrap();
            let segments: Vec<&str> = path
                .segments
                .iter()
                .map(|segment| segment.ident.view(&self.source.data))
                .collect();

            petal.kind = PetalKind::File(path, parent_path.to_path_buf());

            let mut found = false;
            let file = self
                .petal_stack
                .iter()
                .collect::<PathBuf>()
                .join(segments.iter().collect::<PathBuf>());

            for f_path in &[
                file.with_extension(config::HYC_FILE_EXT),
                file.join(config::HYC_DIR_PETAL_FILE),
            ] {
                let path_buf = parent_path.join(&f_path);

                found = match std::fs::exists(&path_buf) {
                    Ok(res) => res,
                    Err(err) => panic!("an error occurred: {err:?}"),
                };

                if found {
                    let PetalKind::File(_, buf) = &mut petal.kind else {
                        break;
                    };

                    *buf = path_buf;
                    break;
                }
            }

            if !found {
                Err(Some(ParserDiag::error(
                    span,
                    ParserDiagErrorKind::UnrecognizedPetalFile { path: file },
                )))?;
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
            ty = Some(dbg!(self.parse_ty())?)
        }

        // =
        let mut val = Option::<Expr>::None;
        if self.expect_exact_nonlf(TokenKind::Eq).0 {
            // EXPR
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
