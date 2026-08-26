use hycc_ast::{
    generic::{GenericParam, GenericParamKind, GenericParamList},
    token::TokenKind,
};

use crate::parser::{Parser, diag::ParseResult, path::PathKind};

impl<'s> Parser<'s> {
    // < GENERIC_PARAM (, GENERIC_PARAM) >
    pub fn parse_generic_params(&mut self) -> ParseResult<GenericParamList> {
        let Some(op_delim) = self.next_nonlf_token() else {
            return Err(None);
        };

        let mut param_decls = GenericParamList {
            list: Vec::new(),
            span: op_delim.span,
        };

        let mut expect = true;
        while !self.expect_exact_nonlf(TokenKind::Greater).0 {
            if !expect {
                self.require_exact_nonlf(TokenKind::SemiColon)?;
            }

            if expect {
                param_decls.list.push(self.parse_generic_param()?);
                expect = false;
            }

            if !expect && self.expect_exact_nonlf(TokenKind::SemiColon).0 {
                expect = true;
                continue;
            }
        }

        Ok(param_decls)
    }

    // GENERIC_PARAM
    // ( RAW_IDENT (: intf_REQS)? ) | ( const RAW_IDENT : TYPE )
    // ( RAW_IDENT (: intf_REQ (, intf_REQ)* ) ) | ( const RAW_IDENT : TYPE )
    pub fn parse_generic_param(&mut self) -> ParseResult<GenericParam> {
        // RAW_IDENT
        let param = self.parse_raw_ident()?;

        // :
        let mut intf_reqs = Vec::new();
        if self.expect_exact_nonlf(TokenKind::Colon).0 {
            let mut expect = true;
            while !self.expect_preserved_exact_nonlf(TokenKind::SemiColon).0
                && !self.expect_preserved_exact_nonlf(TokenKind::Greater).0
            {
                if !expect {
                    self.require_exact_nonlf(TokenKind::Comma)?;
                }

                if expect {
                    // TODO: use (and make) parse_intf_ident or allow associated
                    // types to be parsed in the identifier arguments
                    intf_reqs.push(self.parse_path(PathKind::Ty)?);
                    expect = false;
                }

                if !expect && self.expect_exact_nonlf(TokenKind::Comma).0 {
                    expect = true;
                    continue;
                }
            }
        }

        Ok(GenericParam {
            ident: param,
            intf_reqs,
            kind: GenericParamKind::Ty, // TODO
        })
    }
}
