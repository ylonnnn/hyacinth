use hycc_ast::{
    token::{Token, TokenGraph, TokenIdentKind, TokenKind},
    token_stream::TokenConsumptionKind,
};
use hycc_diagnostic::DiagnosticContext;

use crate::{errors, parser::Parser};

impl<'d, 's> Parser<'d, 's> {
    pub fn parse_raw_ident(&mut self) -> Option<Token> {
        Some(
            self.require_similar_nonlf(TokenKind::Ident(TokenIdentKind::Normal))?
                .underlying()?
                .clone(),
        )
    }
}
