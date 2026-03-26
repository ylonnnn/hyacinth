use hycc_ast::{Expr, token::TokenKind};

use crate::parser::Parser;

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

    pub fn parse_expr(&mut self, min_bp: u8) -> Option<Expr> {
        let Some(mut prefix) = self.parse_prefix_expr() else {
            return None;
        };

        while !self.stream.at_eof() {
            if let Some((lbp, _)) =
                Self::expr_infix_binding_power_of(self.peek_nonlf()?.underlying()?.kind)
                && min_bp > lbp
            {
                break;
            };

            let Some(infix) = self.parse_infix_expr() else {
                break;
            };

            prefix = infix;
        }

        Some(prefix)
    }

    pub fn parse_prefix_expr(&mut self) -> Option<Expr> {
        let token = self.peek_nonlf()?.underlying()?;

        match token.kind {
            TokenKind::Int { .. }
            | TokenKind::Float { .. }
            | TokenKind::Bool
            | TokenKind::Char { .. }
            | TokenKind::String { .. } => {
                todo!("parse literals")
            }

            TokenKind::Ident(..) => {
                todo!("parse identifiers")
            }

            _ => {
                // TODO: throw an error: unexpected token or something
                None
            }
        }
    }

    pub fn parse_infix_expr(&mut self) -> Option<Expr> {
        let token = self.peek_nonlf()?.underlying()?;

        match token.kind {
            _ => {
                // TODO: throw an error: unexpected token or something
                None
            }
        }
    }
}
