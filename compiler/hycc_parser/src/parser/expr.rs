use hycc_ast::{
    Expr, ExprKind, Mutability, Path,
    expr::{ArrayExpr, RefExpr, StructExpr, StructExprField},
    token::{Token, TokenGraph, TokenIdentKind, TokenKind},
    token_stream::{TokenConsumptionKind, TokenMatchExpectation, TokenStream},
};
use hycc_util::ternary;

use crate::parser::{Parser, diag::ParserDiag, parser::ParseResult, path::PathKind};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ExprInfixBindingPower {
    Default,
    Assign,
    Logical,
    Rel,
    Bitwise,
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
            TokenKind::LeftBrace => Some((Default as u8, Default as u8)),

            TokenKind::PlusEq
            | TokenKind::MinusEq
            | TokenKind::StarEq
            | TokenKind::SlashEq
            | TokenKind::PercentEq => Some((Assign as u8, Assign as u8)),

            TokenKind::AmpersandAmpersand | TokenKind::PipePipe => {
                Some((Logical as u8, Logical as u8))
            }

            TokenKind::Eq
            | TokenKind::BangEq
            | TokenKind::Less
            | TokenKind::LessEq
            | TokenKind::Greater
            | TokenKind::GreaterEq => Some((Rel as u8, Rel as u8)),

            TokenKind::Tilde | TokenKind::Ampersand | TokenKind::Pipe => {
                Some((Bitwise as u8, Bitwise as u8))
            }

            TokenKind::LessLess | TokenKind::GreaterGreater => {
                Some((BitShift as u8, BitShift as u8))
            }

            TokenKind::Plus | TokenKind::Minus => Some((Add as u8, Add as u8)),
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some((Mul as u8, Mul as u8)),

            TokenKind::Bang | TokenKind::PlusPlus | TokenKind::MinusMinus => {
                Some((Unary as u8, Unary as u8))
            }

            TokenKind::LeftParen => Some((FnCall as u8, Default as u8)),

            TokenKind::Dot => Some((MemAccess as u8, MemAccess as u8)),

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
                &[],
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
                Some((_, rbp)) if min_bp <= rbp => rbp,
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

            TokenKind::Ident(..) => {
                let path = self.parse_path(PathKind::Expr)?;

                if let Some(tailing) = self.peek_nonlf_token()
                    && tailing.kind == TokenKind::LeftBrace
                {
                    let strct = self.parse_struct_expr(path)?;
                    Ok(Expr::new(ExprKind::Struct(Box::new(strct))))
                } else {
                    Ok(Expr::new(ExprKind::Path(Box::new(path))))
                }
            }

            TokenKind::Ampersand => {
                let span = self.next_nonlf_token().unwrap().span;
                let mutability = if self
                    .expect_exact_nonlf(TokenKind::Ident(TokenIdentKind::Mut))
                    .0
                {
                    Mutability::Mutable
                } else {
                    Mutability::Immutable
                };

                let expr = self.parse_expr(0)?;

                Ok(Expr::new(ExprKind::RefExpr(Box::new(RefExpr {
                    span: span.merge(&expr.span),
                    expr: Box::new(expr),
                    mutability,
                }))))
            }

            TokenKind::LeftBrace => Ok(Expr::new(ExprKind::Array(Box::new(
                self.parse_array_expr()?,
            )))),

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
            | TokenKind::AmpersandAmpersand
            | TokenKind::PipePipe
            | TokenKind::Bang
            | TokenKind::LessLess
            | TokenKind::Ampersand
            | TokenKind::Pipe
            | TokenKind::Tilde
            | TokenKind::Caret => {
                let Some(token) = self.next_nonlf_token() else {
                    return Err((left, None)); // TODO: throw error: missing right-hand side expression
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

            TokenKind::Greater => {
                // This is done in order for the identifier arguments to be properly parsed and
                // since expressions which contain a combination of `>` and another character to
                // the left of it is not exactly common, this may be the more efficient solution
                // rather than manually checking the closing angle brackets of the identifier
                // arguments.

                // TODO: potentially improve this and create a method in the TokenKind for merging
                // mergeable kinds such as these.
                // TokenKind::GreaterEq | TokenKind::GreaterGreater
                let Some(lead) = self.next_nonlf_token() else {
                    unreachable!()
                };

                let Some(trailing) = self.peek_nonlf_token() else {
                    return Err((left, None));
                };

                let trailing = trailing.clone();
                let span = lead.span.merge(&trailing.span);

                let token = match &trailing.kind {
                    TokenKind::Eq | TokenKind::Greater => {
                        self.stream.adjust();
                        Token::new(
                            match &trailing.kind {
                                TokenKind::Eq => TokenKind::GreaterEq,
                                TokenKind::Greater => TokenKind::GreaterGreater,
                                _ => unreachable!(),
                            },
                            span,
                        )
                    }

                    _ => lead,
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
                    return Err((left, None)); // TODO: throw error: missing right-hand side expression
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

    pub fn parse_array_expr(&mut self) -> ParseResult<ArrayExpr> {
        let Some(tg) = self.next_nonlf() else {
            unreachable!()
        };

        let span = tg.span();
        let TokenGraph::Collection { data, .. } = tg else {
            unreachable!()
        };

        let n = data.len();

        self.use_stream(
            TokenStream::new(data.into_iter().skip(1).take(n - 2).collect()),
            |s| -> ParseResult<ArrayExpr> {
                let mut array = ArrayExpr {
                    elements: Vec::new(),
                    span,
                };
                let mut expect = true;

                while !s.eos() {
                    if !expect {
                        s.require_exact_nonlf(TokenKind::Comma)?;
                    }

                    if expect {
                        array.elements.push(s.parse_expr(0)?);
                        expect = false;
                    }

                    if !expect && s.expect_exact_nonlf(TokenKind::Comma).0 {
                        expect = true;
                        continue;
                    }
                }

                Ok(array)
            },
        )
    }

    // PATH { FIELD* }
    // PATH { (IDENT : EXPR)* }
    pub fn parse_struct_expr(&mut self, path: Path) -> ParseResult<StructExpr> {
        let Some(tg) = self.next_nonlf() else {
            unreachable!()
        };

        let span = path.span.merge(&tg.span());
        let TokenGraph::Collection { data, .. } = tg else {
            unreachable!()
        };

        let n = data.len();
        let mut strct = StructExpr {
            path,
            fields: Vec::new(),
            span,
        };

        self.use_stream(
            TokenStream::new(data.into_iter().skip(1).take(n - 2).collect()),
            |s| -> ParseResult<()> {
                let mut expect = true;
                while !s.eos() {
                    if !expect {
                        s.require_exact_nonlf(TokenKind::Comma)?;
                    }

                    if expect {
                        strct.fields.push(s.parse_struct_expr_field()?);
                        expect = false;
                    }

                    if !expect && s.expect_exact_nonlf(TokenKind::Comma).0 {
                        expect = true;
                        continue;
                    }
                }

                Ok(())
            },
        )?;

        Ok(strct)
    }

    // IDENT : EXPR
    pub fn parse_struct_expr_field(&mut self) -> ParseResult<StructExprField> {
        // IDENT
        let ident = self.parse_raw_ident()?;

        // :
        self.require_abs_exact_nonlf(TokenKind::Colon)?;

        // EXPR
        let val = Box::new(self.parse_expr(0)?);

        Ok(StructExprField { ident, val })
    }
}
