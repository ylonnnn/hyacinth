use hycc_ast::{
    Expr, Item, ItemKind, Ty,
    item::{Fn, FnParam, FnParamList, VarDecl},
    token::{TokenGraph, TokenIdentKind, TokenKind},
    token_stream::TokenStream,
};

use crate::parser::{
    Parser,
    diag::{ParserDiag, ParserDiagErrorKind},
    parser::ParseResult,
};

impl Parser {
    pub fn parse_item_with_recovery(&mut self) -> ParseResult<Item> {
        let item = self.parse_item();
        if let Err(_) = item {
            self.sync(vec![TokenKind::LnFeed, TokenKind::RightBrace]);
        }

        item
    }

    pub fn try_parse_item_with_recovery(&mut self) -> ParseResult<Item> {
        let item = self.try_parse_item();
        if let Err(_) = item {
            self.sync(vec![TokenKind::LnFeed, TokenKind::RightBrace]);
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

    pub fn parse_fn_with_recovery(&mut self) -> ParseResult<Fn> {
        let data = self.parse_fn();
        self.try_sync(vec![TokenKind::RightBrace]);

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
            TokenStream::new(data.into_iter().skip(1).take(n - 2).collect()),
            |s| {
                let mut params = FnParamList {
                    span: span,
                    list: Vec::new(),
                };

                if s.stream.is_empty() {
                    return Ok(params);
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
        self.try_sync(vec![TokenKind::LnFeed]);

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
