use hycc_ast::{
    Expr, ExprKind,
    token::TokenKind,
    token_stream::{TokenConsumptionKind, TokenMatchExpectation},
};
use hycc_util::ternary;

use crate::parser::{Parser, diag::ParserDiag, parser::ParseResult};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ExprInfixBindingPower {
    Default,
    Assign,
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

impl<'s> Parser<'s> {
    pub fn expr_infix_binding_power_of(kind: TokenKind) -> Option<(u8, u8)> {
        use ExprInfixBindingPower::*;

        match kind {
            TokenKind::PlusEq
            | TokenKind::MinusEq
            | TokenKind::StarEq
            | TokenKind::SlashEq
            | TokenKind::PercentEq => Some((Assign as u8, Assign as u8)),

            TokenKind::Plus | TokenKind::Minus => Some((Add as u8, Add as u8)),
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some((Mul as u8, Mul as u8)),

            TokenKind::Int { .. }
            | TokenKind::Float { .. }
            | TokenKind::Bool
            | TokenKind::Char { .. }
            | TokenKind::String { .. }
            | TokenKind::Ident(..) => Some((Primary as u8, Primary as u8)),

            _ => None,
        }
    }

    pub fn try_parse_expr_stmt(&mut self) -> ParseResult<Expr> {
        let expr = match self.parse_expr(0) {
            Ok(expr) => expr,
            Err(_) => Err(None)?,
        };

        let is_lf = self
            .stream
            .expect(
                TokenKind::LnFeed,
                TokenConsumptionKind::UponSuccess,
                Vec::new(),
                TokenMatchExpectation::Exact,
            )
            .0;

        ternary!(is_lf, Ok(expr), Err(None))
    }

    pub fn parse_expr_stmt(&mut self) -> ParseResult<Expr> {
        let expr = self.parse_expr(0)?;
        self.require_terminator()?;

        Ok(expr)
    }

    pub fn parse_expr(&mut self, min_bp: u8) -> ParseResult<Expr> {
        let Ok(mut prefix) = self.parse_prefix_expr() else {
            return Err(None);
        };

        while !self.stream.at_eof() {
            let Some(tok) = self.peek_nonlf_token() else {
                return Err(None);
            };

            let rbp = match Self::expr_infix_binding_power_of(tok.kind) {
                Some((_, rbp)) if min_bp < rbp => rbp,
                _ => break,
            };

            match self.parse_infix_expr(prefix, rbp) {
                Ok(infix) => prefix = infix,
                Err((left, _)) => {
                    prefix = left;
                    break;
                }
            };
        }

        Ok(prefix)
    }

    pub fn parse_prefix_expr(&mut self) -> ParseResult<Expr> {
        let Some(token) = self.peek_nonlf_token() else {
            return Err(None);
        };

        match token.kind {
            TokenKind::Int { .. }
            | TokenKind::Float { .. }
            | TokenKind::Bool
            | TokenKind::Char { .. }
            | TokenKind::String { .. } => Ok(Expr::new(ExprKind::Literal(
                self.next_nonlf_token().unwrap().clone(),
            ))),

            TokenKind::Ident(..) => Ok(Expr::new(ExprKind::Path(Box::new(self.parse_path()?)))),

            _ => Err(Some(ParserDiag::unexpected_token_expected_arbitrary(
                token.clone(),
                "expr prefix",
            ))),
        }
    }

    pub fn parse_infix_expr(
        &mut self,
        left: Expr,
        min_bp: u8,
    ) -> ParseResult<Expr, (Expr, Option<ParserDiag>)> {
        let Some(token) = self.peek_nonlf_token() else {
            return Err((left, None));
        };

        match token.kind {
            TokenKind::Int { .. }
            | TokenKind::Float { .. }
            | TokenKind::Bool
            | TokenKind::Char { .. }
            | TokenKind::String { .. }
            | TokenKind::Ident(..) => Err((left, None)),

            TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Percent
            | TokenKind::EqEq
            | TokenKind::BangEq
            | TokenKind::Less
            | TokenKind::LessEq
            | TokenKind::Greater
            | TokenKind::GreaterEq
            | TokenKind::AmpersandAmpersand
            | TokenKind::PipePipe
            | TokenKind::Bang
            | TokenKind::LessLess
            | TokenKind::GreaterGreater
            | TokenKind::Ampersand
            | TokenKind::Pipe
            | TokenKind::Tilde
            | TokenKind::Caret => {
                let Some(token) = self.next_nonlf_token() else {
                    return Err((left, None));
                };

                let right = match self.parse_expr(min_bp) {
                    Ok(right) => right,
                    Err(matched) => return Err((left, matched)),
                };

                Ok(Expr::new(ExprKind::Binary(
                    token,
                    Box::new(left),
                    Box::new(right),
                )))
            }

            TokenKind::Eq
            | TokenKind::PlusEq
            | TokenKind::MinusEq
            | TokenKind::StarEq
            | TokenKind::SlashEq
            | TokenKind::PercentEq => {
                if self.next_nonlf_token().is_none() {
                    return Err((left, None));
                };

                let right = match self.parse_expr(min_bp) {
                    Ok(right) => right,
                    Err(diag) => return Err((left, diag)),
                };

                Ok(Expr::new(ExprKind::Assign(Box::new(left), Box::new(right))))
            }

            _ => Err((
                left,
                Some(ParserDiag::unexpected_token_expected_arbitrary(
                    token.clone(),
                    "expr infix operation",
                )),
            )),
        }
    }
}
