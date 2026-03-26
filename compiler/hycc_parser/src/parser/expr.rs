use hycc_ast::{Expr, token::TokenKind};

use crate::parser::{Parser, parser::ParseResult};

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

            if let Some((lbp, _)) = Self::expr_infix_binding_power_of(tok.kind)
                && min_bp > lbp
            {
                break;
            };

            let Ok(infix) = self.parse_infix_expr() else {
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

            // TokenKind::Ident(..) => {
            //     todo!("parse identifiers")
            // }
            _ => {
                // TODO: throw an error: unexpected token or something
                Err(false)
            }
        }
    }

    pub fn parse_infix_expr(&mut self) -> ParseResult<Expr> {
        let Some(token) = self.peek_nonlf_token() else {
            return Err(false);
        };

        match token.kind {
            _ => {
                // TODO: throw an error: unexpected token or something
                Err(false)
            }
        }
    }
}
