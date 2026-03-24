use hycc_ast::{
    Item, ItemKind,
    item::{Fn, FnParam, FnParamList},
    token::{TokenGraph, TokenIdentKind, TokenKind},
    token_stream::TokenStream,
};

use crate::parser::Parser;

impl<'d, 's> Parser<'d, 's> {
    pub fn parse_item(&mut self) -> Option<Item> {
        let Some(tok) = self.stream.peek()?.underlying() else {
            return None;
        };

        dbg!(tok.kind);
        let item_kind = match tok.kind {
            TokenKind::Ident(TokenIdentKind::Fn) => {
                Some(ItemKind::Fn(self.parse_fn_with_recovery()?))
            }
            _ => {
                self.stream.adjust();
                println!("throw error: unexpected [tok], expected an item");
                None
            }
        }?;

        Some(Item::new(item_kind))
    }

    pub fn parse_fn_with_recovery(&mut self) -> Option<Fn> {
        let data = self.parse_fn();
        if self.dctx.is_in_disarray() {
            todo!();
        }

        data
    }

    // fn IDENT(GENERIC_PARAMS)?((PARAM_LIST)?) RET_TY? BLOCK
    // fn IDENT < GENERIC_PARAM (, GENERIC_PARAM)* > ( PARAM (, PARAM)? ) RET_TY? B{ STMT* }
    pub fn parse_fn(&mut self) -> Option<Fn> {
        // fn
        self.stream.adjust();

        // IDENT
        let ident = self.parse_raw_ident();

        // (PARAM (, PARAM)*)
        let params = self.parse_fn_param_list();

        // TODO: parse ret type

        // { STMT* }
        let body = self.parse_block();

        dbg!(&ident);
        dbg!(&body);

        Some(Fn {
            ident: ident?,
            params: params?,
            ret_ty: None,
            body: body?,
        })
    }

    // (PARAM (, PARAM)*)
    pub fn parse_fn_param_list(&mut self) -> Option<FnParamList> {
        println!("TODO: parse function param list");

        let TokenGraph::Collection { data, .. } = self.require_exact_nonlf(TokenKind::LeftParen)?
        else {
            return None;
        };

        self.use_stream(TokenStream::new(data), |s| {
            // TODO: parse function params

            println!("{}", s.stream);
            None
        })
    }

    pub fn parse_fn_param(&mut self) -> Option<FnParam> {
        todo!()
    }
}
