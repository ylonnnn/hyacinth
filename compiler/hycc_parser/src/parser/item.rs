use hycc_ast::{
    Mutability,
    expr::Expr,
    item::{
        Extend, Fn, FnParam, FnParamList, FnSig, ItemAccessibility, ItemLevel, Petal, PetalKind,
        Proto, ProtoItem, ProtoItemAssocFnKind, PubAccessibilityKind, Refer, ReferTarget,
        ReferTargetKind, Struct, StructField, StructFieldAccessibility, StructFieldList, VarDecl,
    },
    item::{Item, ItemKind},
    token::{TokenGraph, TokenIdentKind, TokenKind},
    token_stream::TokenStream,
    ty::Ty,
};
use hycc_diagnostic::DiagnosticContext;
use hycc_session::config;
use hycc_util::ternary;
use std::path::{self, PathBuf};

use crate::parser::{
    Parser,
    diag::{ParserDiag, ParserDiagErrorKind},
    parser::{ParseResult, ParserTerminatorKind},
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

            TokenKind::Ident(TokenIdentKind::Refer) => {
                ItemKind::Refer(Box::new(self.parse_refer_with_recovery()?))
            }

            TokenKind::Ident(TokenIdentKind::Petal) => {
                ItemKind::Petal(Box::new(self.parse_petal_with_recovery()?))
            }

            TokenKind::Ident(TokenIdentKind::Proto) => {
                ItemKind::Proto(Box::new(self.parse_proto_with_recovery()?))
            }

            TokenKind::Ident(TokenIdentKind::Extend) => {
                ItemKind::Extend(Box::new(self.parse_extend_with_recovery()?))
            }

            TokenKind::Ident(TokenIdentKind::Struct) => {
                ItemKind::Struct(Box::new(self.parse_struct_with_recovery()?))
            }

            TokenKind::Ident(TokenIdentKind::Fn) => {
                ItemKind::Fn(Box::new(self.parse_fn_with_recovery()?))
            }

            TokenKind::Ident(TokenIdentKind::Let) => {
                ItemKind::VarDecl(Box::new(self.parse_var_decl_with_recovery()?))
            }

            _ => Err(None)?,
        };

        let mut item = Item::new(
            kind,
            ternary!(
                self.depth == 0,
                ItemLevel::Top,
                ItemLevel::Local(self.depth.saturating_sub(1))
            ),
        );
        item.span = span.merge(item.span);

        Ok(item)
    }

    pub fn parse_item_with_accessibility(&mut self) -> ParseResult<Item> {
        let mut pub_kind = PubAccessibilityKind::All;
        if let (true, Some(tokg)) = self.expect_exact_nonlf(TokenKind::LeftBracket) {
            let TokenGraph::Collection { data, .. } = tokg else {
                unreachable!()
            };

            let n = data.len();
            pub_kind = self.use_stream(
                TokenStream::new(data.into_iter().skip(1).take(n - 2).collect()),
                |s| -> ParseResult<PubAccessibilityKind> {
                    let Some(tok) = s.peek_nonlf_token() else {
                        Err(None)?
                    };

                    match &tok.kind {
                        TokenKind::Ident(TokenIdentKind::Super) => Ok(PubAccessibilityKind::Super),
                        TokenKind::Ident(TokenIdentKind::Spathe) => {
                            Ok(PubAccessibilityKind::Spathe)
                        }

                        _ => Err(Some(ParserDiag::unexpected_token_expected_arbitrary(
                            tok.clone(),
                            "public access modifier kind",
                        )))?,
                    }
                },
            )?;
        }

        let mut item = self.parse_item_with_recovery()?;
        item.accessibility = ItemAccessibility::Pub(pub_kind);

        Ok(item)
    }

    pub fn parse_refer_with_recovery(&mut self) -> ParseResult<Refer> {
        let data = self.parse_refer();
        self.try_sync(&[TokenKind::RightBrace, TokenKind::LnFeed]);

        data
    }

    pub fn parse_refer(&mut self) -> ParseResult<Refer> {
        let target = self.parse_refer_target()?;
        self.require_terminator(ParserTerminatorKind::Both)?;

        Ok(Refer {
            span: target.span,
            target,
        })
    }

    pub fn parse_refer_target(&mut self) -> ParseResult<ReferTarget> {
        let symbol = self.parse_ident(PathKind::Ty)?;
        let Some(tok) = self.peek_nonlf_token() else {
            todo!()
        };

        let kind = match tok.kind {
            TokenKind::Ident(TokenIdentKind::As) => {
                self.adjust_to_nonlf();
                let alias = self.parse_raw_ident()?;
                ReferTargetKind::Child(Some(alias))
            }

            TokenKind::ColonColon => {
                self.adjust_to_nonlf();

                // TODO: *

                // [
                if let (true, Some(tokg)) = self.expect_exact_nonlf(TokenKind::LeftBracket) {
                    // TODO: multi-target parsing
                    let TokenGraph::Collection { data, .. } = tokg else {
                        unreachable!()
                    };

                    let n = data.len();
                    let children = self.use_stream(
                        TokenStream::new(data.into_iter().skip(1).take(n - 2).collect()),
                        |s| -> ParseResult<Vec<ReferTarget>> {
                            let mut children = Vec::new();
                            let mut expect = true;

                            while !s.eos() {
                                if !expect {
                                    s.require_exact_nonlf(TokenKind::Comma)?;
                                }

                                if expect {
                                    children.push(s.parse_refer_target()?);
                                    expect = false;
                                }

                                if !expect && s.expect_exact_nonlf(TokenKind::Comma).0 {
                                    expect = true;
                                    continue;
                                }
                            }

                            Ok(children)
                        },
                    )?;

                    ReferTargetKind::Parent(children)
                }
                // IDENT
                else {
                    ReferTargetKind::Parent(vec![self.parse_refer_target()?])
                }
            }

            _ => ReferTargetKind::Child(None),
        };

        // let span = match &kind {
        //     ReferTargetKind::Child(alias) => alias.map(|alias| alias.span).unwrap_or(symbol.span),
        //     ReferTargetKind::Parent()
        // };

        Ok(ReferTarget {
            span: symbol.span,
            kind,
            symbol,
        })
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

                    while !s.eos() {
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

        if self.depth > 0 && matches!(petal.kind, PetalKind::File(..)) {
            Err(Some(ParserDiag::error(
                span,
                ParserDiagErrorKind::IllegalLocalNonInlinePetalDeclaration,
            )))?;
        }

        self.require_terminator(ParserTerminatorKind::LnFeed)?;

        Ok(petal)
    }

    pub fn parse_proto_with_recovery(&mut self) -> ParseResult<Proto> {
        let data = self.parse_proto();
        self.try_sync(&[TokenKind::RightBrace]);

        data
    }

    // proto IDENT (GENERIC_PARAMS)? { PROTO_ITEM* }
    // proto IDENT (< GENERIC_PARAM (, GENERIC_PARAM)* >)? { PROTO_ITEM* }
    pub fn parse_proto(&mut self) -> ParseResult<Proto> {
        // IDENT
        let ident = self.parse_raw_ident()?;

        // TODO: generic params

        // { PROTO_ITEM* }
        let data = match self.require_exact_nonlf(TokenKind::LeftBrace)? {
            TokenGraph::Collection { data, .. } => data,
            _ => Err(None)?,
        };

        let n = data.len();
        let items = self.use_stream(
            TokenStream::new(data.into_iter().skip(1).take(n - 2).collect()),
            |s| -> Vec<ProtoItem> {
                let mut items = Vec::new();
                if s.stream.is_empty() {
                    return items;
                }

                while !s.eos() {
                    match s.parse_proto_item() {
                        Ok(item) => items.push(item),
                        Err(diag) => {
                            diag.map(|diag| s.dctx.add(diag));
                        }
                    }
                }

                items
            },
        );

        // todo!("parse proto")
        Ok(Proto {
            span: ident.span,
            ident,
            items,
        })
    }

    pub fn parse_proto_item(&mut self) -> ParseResult<ProtoItem> {
        let Some(tok) = self.next_nonlf_token() else {
            return Err(None);
        };

        Ok(match tok.kind {
            TokenKind::Ident(TokenIdentKind::Fn) => {
                let sig = self.parse_fn_sig(false)?;
                let kind = if self.expect_preserved_similar_nonlf(TokenKind::LeftBrace).0 {
                    ProtoItemAssocFnKind::Impl(Box::new(Item::new(
                        ItemKind::Fn(Box::new(Fn {
                            sig,
                            body: self.parse_block()?,
                        })),
                        ternary!(
                            self.depth == 0,
                            ItemLevel::Top,
                            ItemLevel::Local(self.depth.saturating_sub(1))
                        ),
                    )))
                } else {
                    self.require_terminator(ParserTerminatorKind::Both)?;
                    ProtoItemAssocFnKind::Sig(Box::new(sig))
                };

                ProtoItem::AssocFn(kind)
            }

            TokenKind::Ident(TokenIdentKind::Let) => ProtoItem::AssocConst(Box::new(Item::new(
                ItemKind::VarDecl(Box::new(self.parse_var_decl_with_recovery()?)),
                ternary!(
                    self.depth == 0,
                    ItemLevel::Top,
                    ItemLevel::Local(self.depth.saturating_sub(1))
                ),
            ))),

            // TODO: associated types
            _ => Err(None)?,
        })
    }

    pub fn parse_extend_with_recovery(&mut self) -> ParseResult<Extend> {
        let data = self.parse_extend();
        self.try_sync(&[TokenKind::LeftBrace]);

        data
    }

    // extend < GENERIC_PARAMS > TY { ITEM* }
    // extend < GENERIC_PARAM (, GENERIC_PARAM)* > TY { ITEM* }
    pub fn parse_extend(&mut self) -> ParseResult<Extend> {
        let generic_params = ternary!(
            self.expect_preserved_exact_nonlf(TokenKind::Less).0,
            Some(self.parse_generic_params()?),
            None
        );

        // TY
        let target = self.parse_ty()?;

        let data = match self.require_exact_nonlf(TokenKind::LeftBrace)? {
            TokenGraph::Collection { data, .. } => data,
            _ => Err(None)?,
        };

        let mut extend = Extend {
            target,
            generic_params,
            items: Vec::new(),
        };
        let n = data.len();

        self.use_stream(
            TokenStream::new(data.into_iter().skip(1).take(n - 2).collect()),
            |s| {
                if s.stream.is_empty() {
                    return;
                }

                while !s.eos() {
                    match s.parse_extend_item() {
                        Ok(item) => extend.items.push(item),
                        Err(diag) => {
                            if let Some(diag) = diag {
                                s.dctx.add(diag);
                            }
                        }
                    }
                }
            },
        );

        Ok(extend)
    }

    pub fn parse_extend_item(&mut self) -> ParseResult<Item> {
        let item = self.parse_item_with_recovery()?;
        if matches!(&item.kind, ItemKind::Fn(_)) {
            return Ok(item);
        }

        Err(Some(ParserDiag::error(
            item.span,
            ParserDiagErrorKind::UnsupportedItem {
                item_kind: item.kind,
                context: "type extension",
            },
        )))
    }

    pub fn parse_struct_with_recovery(&mut self) -> ParseResult<Struct> {
        let data = self.parse_struct();
        self.try_sync(&[TokenKind::RightBrace]);

        data
    }

    // struct IDENT < GENERIC_PARAMS > { (FIELD (, FIELD)?)* }
    // struct IDENT < GENERIC_PARAM (, GENERIC_PARAM)* > { (IDENT : TY (, IDENT : TY)?)* }
    pub fn parse_struct(&mut self) -> ParseResult<Struct> {
        // IDENT
        let ident = self.parse_raw_ident()?;

        // GENERIC_PARAMS
        let generic_params = ternary!(
            self.expect_preserved_exact_nonlf(TokenKind::Less).0,
            Some(self.parse_generic_params()?),
            None
        );

        // FIELDS
        let fields = self.parse_struct_fields()?;
        if fields.list.len() > config::HYC_STRUCT_FIELD_LIMIT {
            Err(Some(ParserDiag::error(
                fields.span,
                ParserDiagErrorKind::InvalidStructFieldCount(fields.list.len() as u8),
            )))
        } else {
            Ok(Struct {
                ident,
                generic_params,
                fields,
            })
        }
    }

    // { (FIELD (, FIELD)?)* }
    // { (IDENT : TY (, IDENT : TY)?)* }
    pub fn parse_struct_fields(&mut self) -> ParseResult<StructFieldList> {
        let Ok(tokg) = self.require_exact_nonlf(TokenKind::LeftBrace) else {
            return Err(None);
        };

        let span = tokg.span();
        let TokenGraph::Collection { data, .. } = tokg else {
            unreachable!()
        };

        let n = data.len();

        self.use_stream(
            TokenStream::new(data.into_iter().skip(1).take(n - 2).collect()),
            |s| -> ParseResult<StructFieldList> {
                let mut fields = StructFieldList {
                    list: Vec::new(),
                    span,
                };

                let mut expect = true;
                while !s.eos() {
                    if !expect {
                        s.require_exact_nonlf(TokenKind::Comma)?;
                    }

                    if expect {
                        fields.list.push(s.parse_struct_field()?);
                        expect = false;
                    }

                    if !expect && s.expect_exact_nonlf(TokenKind::Comma).0 {
                        expect = true;
                        continue;
                    }
                }

                Ok(fields)
            },
        )
    }

    // (ACCESSIBILITY)? IDENT : TY
    pub fn parse_struct_field(&mut self) -> ParseResult<StructField> {
        // TODO: ACCESSIBILITY

        // IDENT
        let ident = self.parse_raw_ident()?;

        // :
        self.require_abs_exact_nonlf(TokenKind::Colon)?;

        // TY
        let ty = Box::new(self.parse_ty()?);

        Ok(StructField {
            ident,
            ty,
            accessibility: StructFieldAccessibility::Priv, // TODO: default for now
        })
    }

    pub fn parse_fn_with_recovery(&mut self) -> ParseResult<Fn> {
        let data = self.parse_fn();
        self.try_sync(&[TokenKind::RightBrace]);

        data
    }

    // fn IDENT (GENERIC_PARAMS)? ((PARAM_LIST)?) RET_TY?
    // fn IDENT < GENERIC_PARAM (, GENERIC_PARAM)* > ( PARAM (, PARAM)? ) RET_TY?
    pub fn parse_fn_sig(&mut self, require_term: bool) -> ParseResult<FnSig> {
        // IDENT
        let ident = self.parse_raw_ident();

        // GENERIC_PARAMS
        let generic_params = ternary!(
            self.expect_preserved_exact_nonlf(TokenKind::Less).0,
            Some(self.parse_generic_params()?),
            None
        );
        if self.expect_preserved_exact_nonlf(TokenKind::Less).0 {
            self.parse_generic_params()?;
        }

        // (PARAM (, PARAM)*)
        let params = self.parse_fn_param_list()?;

        // ->
        let mut ret_ty = Option::<Ty>::None;
        if self.expect_exact_nonlf(TokenKind::MinusGreater).0 {
            // RET_TY
            ret_ty = Some(self.parse_ty()?);
        }

        if require_term {
            self.require_terminator(ParserTerminatorKind::LnFeed)?;
        }

        Ok(FnSig {
            ident: ident?,
            generic_params,
            params,
            ret_ty: ret_ty.map(Box::new),
        })
    }

    // FN_SIG BLOCK
    // fn IDENT < GENERIC_PARAM (, GENERIC_PARAM)* > ( PARAM (, PARAM)? ) RET_TY? { STMT* }
    pub fn parse_fn(&mut self) -> ParseResult<Fn> {
        // FN SIG
        let sig = self.parse_fn_sig(false);

        // { STMT* }
        let body = self.parse_block();

        self.require_terminator(ParserTerminatorKind::LnFeed)?;

        Ok(Fn {
            sig: sig?,
            body: body?,
        })
    }

    // (PARAM (, PARAM)*)
    pub fn parse_fn_param_list(&mut self) -> ParseResult<FnParamList> {
        let Ok(tokg) = self.require_exact_nonlf(TokenKind::LeftParen) else {
            return Err(None);
        };

        let span = tokg.span();
        let TokenGraph::Collection { data, .. } = tokg else {
            unreachable!()
        };

        let n = data.len();
        self.use_stream(
            TokenStream::new(data.into_iter().skip(1).take(n - 2).collect()),
            |s| {
                let mut params = FnParamList {
                    span: span,
                    list: Vec::new(),
                };

                let mut expect = true;
                while !s.eos() {
                    if !expect {
                        s.require_exact_nonlf(TokenKind::Comma)?;
                    }

                    if expect {
                        params.list.push(s.parse_fn_param()?);
                        expect = false;
                    }

                    if !expect && s.expect_exact_nonlf(TokenKind::Comma).0 {
                        expect = true;
                        continue;
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
    // let mut? IDENT (: TY)? (= EXPR)? (TERM ::= '\n')
    pub fn parse_var_decl(&mut self) -> ParseResult<VarDecl> {
        // mut?
        let mut mutability = Mutability::Immutable;
        if self
            .expect_exact_nonlf(TokenKind::Ident(TokenIdentKind::Mut))
            .0
        {
            mutability = Mutability::Mutable;
        }

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
        if self.expect_exact_nonlf(TokenKind::Eq).0 {
            // EXPR
            val = Some(self.parse_expr(0)?);
        }

        self.require_terminator(ParserTerminatorKind::Both)?;

        // Validate variable declaration
        if (ty.is_none() && val.is_none()) || (self.depth == 0 && (ty.is_none() || val.is_none())) {
            return Err(Some(ParserDiag::error(
                ident.span,
                ParserDiagErrorKind::InvalidVarDecl {
                    ident,
                    depth: self.depth,
                },
            )));
        }

        Ok(VarDecl {
            ident,
            mutability,
            ty: ty.map(Box::new),
            val: val.map(Box::new),
        })
    }
}
