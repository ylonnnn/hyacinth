use hycc_ast::{Expr, ExprKind, token::TokenKind};
use hycc_diagnostic::DiagnosticContext;
use hycc_util::ternary;

use crate::{
    errors,
    parser::{Parser, parser::ParseResult},
};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ExprInfixBindingPower {
    Default,
    Bitwise,
    Rel,
    BitShift,
    Add,
    Mul,
    Exp,
    Unary,
    FnCall,
    MemAccess,
    Primary,
}

impl<'d, 's> Parser<'d, 's> {
    pub fn expr_infix_binding_power_of(kind: TokenKind) -> Option<(u8, u8)> {
        use ExprInfixBindingPower::*;

        match kind {
            TokenKind::Plus | TokenKind::Minus => Some((Add as u8, Add as u8)),
            _ => None,
        }
    }

    pub fn parse_expr(&mut self, min_bp: u8) -> ParseResult<Expr> {
        let Ok(mut prefix) = self.parse_prefix_expr() else {
            return Err(false);
        };

        while !self.stream.at_eof() {
            let Some(tok) = self.peek_nonlf_token() else {
                return Err(false);
            };

            let rbp = match Self::expr_infix_binding_power_of(tok.kind) {
                Some((_, rbp)) => ternary!(min_bp > rbp, break, rbp),
                _ => {
                    break;
                }
            };

            let Ok(infix) = self.parse_infix_expr(&prefix, rbp) else {
                break;
            };

            prefix = infix;
        }

        Ok(prefix)
    }

    pub fn parse_prefix_expr(&mut self) -> ParseResult<Expr> {
        let Some(token) = self.peek_nonlf_token() else {
            return Err(false);
        };

        match token.kind {
            // TokenKind::Int { .. }
            // | TokenKind::Float { .. }
            // | TokenKind::Bool
            // | TokenKind::Char { .. }
            // | TokenKind::String { .. } => {
            //     todo!("parse literals")
            // }
            TokenKind::Ident(..) => match self.parse_path() {
                Ok(expr) => Ok(Expr::new(ExprKind::Path(Box::new(expr)))),
                Err(_) => Err(true)?,
            },

            _ => {
                self.dctx.add(errors::unexpected_token(
                    self.source,
                    &token,
                    Some("expected expr prefix token"),
                ));

                Err(true)
            }
        }
    }

    pub fn parse_infix_expr(&mut self, _left: &Expr, _min_bp: u8) -> ParseResult<Expr> {
        let Some(token) = self.peek_nonlf_token() else {
            return Err(false);
        };

        match token.kind {
            _ => {
                self.dctx.add(errors::unexpected_token(
                    self.source,
                    &token,
                    Some("expected expr infix token"),
                ));

                Err(true)
            }
        }
    }
}
