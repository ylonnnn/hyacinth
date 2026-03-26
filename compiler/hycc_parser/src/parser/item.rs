use hycc_ast::{
    Item, ItemKind,
    item::{Fn, FnParam, FnParamList},
    token::{TokenGraph, TokenIdentKind, TokenKind},
    token_stream::TokenStream,
};
use hycc_diagnostic::DiagnosticContext;

use crate::{
    errors,
    parser::{Parser, parser::ParseResult},
};

impl<'d, 's> Parser<'d, 's> {
    pub fn parse_item(&mut self) -> ParseResult<Item> {
        let Some(tok) = self.peek_nonlf_token() else {
            return Err(false);
        };

        let tok = tok.clone();
        let item = self.try_parse_item();

        // TODO: fix misdiagnosis of errors when item is None,
        //       the issue may be not this part, but rather how
        //       item parsers handle malformed items
        if let Err(matched) = item
            && !matched
        {
            self.dctx.add(errors::unexpected_token(
                self.source,
                &tok,
                Some("expected an item"),
            ));
        }

        item
    }

    pub fn parse_item_with_recovery(&mut self) -> ParseResult<Item> {
        let item = self.parse_item();
        self.try_sync(vec![TokenKind::LnFeed, TokenKind::RightBrace]);

        item
    }

    pub fn try_parse_item(&mut self) -> ParseResult<Item> {
        let Some(tok) = self.next_nonlf_token() else {
            return Err(false);
        };

        let item_kind = match tok.kind {
            TokenKind::Ident(TokenIdentKind::Fn) => {
                Ok(ItemKind::Fn(self.parse_fn_with_recovery()?))
            }
            _ => Err(false),
        }?;

        Ok(Item::new(item_kind))
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

        // TODO: parse ret type

        // { STMT* }
        let body = self.parse_block();

        if let Ok(ident) = &ident {
            println!("ident: {}", ident.view(&self.source.data));
        }
        dbg!(&body);

        Ok(Fn {
            ident: ident?,
            params: params?,
            ret_ty: None,
            body: body?,
        })
    }

    // (PARAM (, PARAM)*)
    pub fn parse_fn_param_list(&mut self) -> ParseResult<FnParamList> {
        println!("TODO: parse function param list");
        let Some(TokenGraph::Collection { data, .. }) =
            self.require_exact_nonlf(TokenKind::LeftParen)
        else {
            return Err(true);
        };

        self.use_stream(TokenStream::new(data), |s| {
            // TODO: parse function params

            println!("{}", s.stream);
            // Some(FnParamList {
            //     span: s.next_nonlf()?.underlying()?.span,
            //     list: Vec::new(),
            // })
            Err(true)
        })
    }

    pub fn parse_fn_param(&mut self) -> ParseResult<FnParam> {
        todo!()
    }
}
