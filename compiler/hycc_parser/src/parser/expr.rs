use hycc_ast::{
    Mutability,
    block::Block,
    expr::{
        AnonFn, AnonFnParam, AnonFnParamList, ArrayExpr, CallArguments, CastExpr, Expr, ExprKind,
        FieldAccess, FnCall, IfExpr, MethodCall, RefExpr, StructExpr, StructExprField, TupleExpr,
        Unary,
    },
    path::{Identifier, Path},
    stmt::{PassStmt, RetStmt, Stmt, StmtKind},
    token::{Token, TokenGraph, TokenIdentKind, TokenKind},
    token_stream::{TokenConsumptionKind, TokenMatchExpectation, TokenStream},
    ty::Ty,
};
use hycc_diagnostic::diagnostic::Diagnostics;
use hycc_span::Span;
use hycc_util::ternary;

use crate::parser::{
    Parser,
    diag::{ParseResult, ParserDiag},
    parser::{ParserCtx, ParserTerminatorKind},
    path::PathKind,
};

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
    Cast,
    Unary,
    FnCall,
    FieldAccess,
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
            TokenKind::CaretCaret => Some((Exp as u8, Exp as u8)),

            TokenKind::Ident(TokenIdentKind::As) => Some((Cast as u8, Cast as u8)),

            TokenKind::Bang => Some((Unary as u8, Unary as u8)),

            TokenKind::LeftParen => Some((FnCall as u8, Default as u8)),

            TokenKind::Dot => Some((FieldAccess as u8, FieldAccess as u8)),

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

        self.require_terminator(ParserTerminatorKind::Both)
            .and_then(|_| Ok(expr))
    }

    pub fn parse_expr_stmt(&mut self) -> ParseResult<Expr> {
        let expr = self.parse_expr(0)?;
        self.require_terminator(ParserTerminatorKind::Both)?;

        Ok(expr)
    }

    pub fn parse_expr(&mut self, min_bp: u8) -> ParseResult<Expr> {
        let mut prefix = match self.parse_prefix_expr() {
            Ok(prefix) => prefix,
            err => return err,
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
                Err((left, diag)) => {
                    prefix = left;

                    if let Some(diag) = diag {
                        self.dctx.add(diag);
                    }

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

            TokenKind::Ident(ident_kind) => {
                let span = token.span;
                match ident_kind {
                    TokenIdentKind::If => {
                        Ok(Expr::new(ExprKind::If(Box::new(self.parse_if_expr()?))))
                    }

                    _ => {
                        let path = self.parse_path(PathKind::Expr)?;
                        let Some(trailing) = self.peek_nonlf_token() else {
                            return Ok(Expr::new(ExprKind::Path(Box::new(path))));
                        };

                        match &trailing.kind {
                            TokenKind::LeftBrace if !matches!(self.ctx, ParserCtx::IfCond) => {
                                Ok(Expr::new(ExprKind::Struct(Box::new(
                                    self.parse_struct_expr(path)?,
                                ))))
                            }

                            TokenKind::LeftParen if ident_kind == TokenIdentKind::Fn => Ok(
                                Expr::new(ExprKind::AnonFn(Box::new(self.parse_anon_fn(span)?))),
                            ),

                            _ => Ok(Expr::new(ExprKind::Path(Box::new(path)))),
                        }
                    }
                }
            }

            TokenKind::Minus | TokenKind::Bang | TokenKind::Star => {
                let Some(tok) = self.next_nonlf_token() else {
                    unreachable!()
                };

                Ok(Expr::new(ExprKind::Unary(Box::new(Unary::Pre(
                    tok,
                    Box::new(self.parse_expr(ExprInfixBindingPower::Unary as u8)?),
                )))))
            }

            TokenKind::Ampersand => Ok(Expr::new(ExprKind::RefExpr(Box::new(
                self.parse_ref_expr()?,
            )))),

            TokenKind::LeftBrace => Ok(Expr::new(ExprKind::Block(Box::new(self.parse_block()?)))),

            TokenKind::LeftBracket => Ok(Expr::new(ExprKind::Array(Box::new(
                self.parse_array_expr()?,
            )))),

            TokenKind::LeftParen => self.parse_paren_enclosed_expr(),

            _ => Err(Some(ParserDiag::unexpected_token_expected_arbitrary(
                token.clone(),
                "expr",
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

        let peek = self.stream.peek();
        match token.kind {
            TokenKind::Int { .. }
            | TokenKind::Float { .. }
            | TokenKind::Bool
            | TokenKind::Char { .. }
            | TokenKind::String { .. } => Err((left, None)),

            TokenKind::Ident(kind) => match kind {
                TokenIdentKind::As => Ok(Expr::new(self.parse_cast_expr(left)?)),
                _ => Err((left, None)),
            },

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
            | TokenKind::Caret => self.parse_binary_expr(left, min_bp),

            TokenKind::Greater => {
                // This is done in order for the identifier arguments to be properly
                // parsed and since expressions which contain a combination of `>`
                // and another character to the left of it is not exactly common, this
                // may be the more efficient solution rather than manually checking
                // the closing angle brackets of the identifier arguments.

                // TODO: potentially improve this and create a method in the TokenKind
                // for merging mergeable kinds such as these.
                // TokenKind::GreaterEq | TokenKind::GreaterGreater
                let Some(lead) = self.next_nonlf_token() else {
                    unreachable!()
                };

                let Some(trailing) = self.peek_nonlf_token() else {
                    return Err((left, None));
                };

                let trailing = trailing.clone();
                let span = lead.span.merge(trailing.span);

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
                    Err(diag) => return Err((left, diag)),
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
            | TokenKind::PercentEq => Ok(Expr::new(self.parse_assign(left)?)),

            TokenKind::LeftParen => {
                if peek.map_or(false, |tokg| {
                    tokg.underlying()
                        .map_or(false, |tok| tok.kind == TokenKind::LnFeed)
                }) {
                    return Err((left, None));
                }

                Ok(Expr::new(ExprKind::FnCall(Box::new(
                    self.parse_fn_call(left)?,
                ))))
            }

            TokenKind::Dot => Ok(Expr::new(self.parse_field_access_or_method_call(left)?)),

            TokenKind::LeftBrace => Err((left, None)),

            _ => Err((
                left,
                Some(ParserDiag::unexpected_token_expected_arbitrary(
                    token.clone(),
                    "expr infix operation",
                )),
            )),
        }
    }

    pub fn parse_binary_expr(
        &mut self,
        left: Expr,
        min_bp: u8,
    ) -> ParseResult<Expr, (Expr, Option<ParserDiag>)> {
        let Some(token) = self.next_nonlf_token() else {
            todo!("throw error: missing right-hand side expression")
        };

        let right = match self.parse_expr(min_bp) {
            Ok(right) => right,
            Err(diag) => return Err((left, diag)),
        };

        Ok(Expr::new(ExprKind::Binary(
            token,
            Box::new(left),
            Box::new(right),
        )))
    }

    pub fn parse_cast_expr(
        &mut self,
        left: Expr,
    ) -> ParseResult<ExprKind, (Expr, Option<ParserDiag>)> {
        self.adjust_to_nonlf();

        match self.parse_ty() {
            Ok(ty) => {
                let span = left.span.merge(ty.span);
                Ok(ExprKind::Cast(Box::new(CastExpr {
                    span,
                    expr: Box::new(left),
                    ty: Box::new(ty),
                })))
            }

            Err(err) => Err((left, err)),
        }
    }

    pub fn parse_assign(
        &mut self,
        left: Expr,
    ) -> ParseResult<ExprKind, (Expr, Option<ParserDiag>)> {
        if self.next_nonlf_token().is_none() {
            todo!("throw error: missing right-hand side expression")
        };

        let right = match self.parse_expr(0) {
            Ok(right) => right,
            Err(diag) => return Err((left, diag)),
        };

        Ok(ExprKind::Assign(Box::new(left), Box::new(right)))
    }

    pub fn parse_paren_enclosed_expr(&mut self) -> ParseResult<Expr> {
        self.use_ctx(ParserCtx::Normal, |s| {
            let tokg = s.require_abs_exact_nonlf(TokenKind::LeftParen)?;
            let span = tokg.span();

            let TokenGraph::Collection { data, .. } = tokg else {
                unreachable!()
            };

            let n = data.len();
            s.use_stream(
                TokenStream::new(data.into_iter().skip(1).take(n - 2).collect()),
                |s| -> ParseResult<Expr> {
                    let mut tup = TupleExpr {
                        elements: Vec::new(),
                        span,
                    };
                    let mut expect = true;

                    while !s.eos() {
                        if !expect {
                            s.require_exact_nonlf(TokenKind::Comma)?;
                        }

                        if expect {
                            tup.elements.push(s.parse_expr(0)?);
                            expect = false;
                        }

                        if !expect && s.expect_exact_nonlf(TokenKind::Comma).0 {
                            expect = true;
                            continue;
                        }
                    }

                    Ok(if tup.elements.is_empty() {
                        todo!("unit expr")
                    } else if tup.elements.len() == 1 {
                        tup.elements.into_iter().next().unwrap()
                    } else {
                        Expr::new(ExprKind::Tuple(Box::new(tup)))
                    })
                },
            )
        })
    }

    pub fn parse_fn_call_arguments(&mut self) -> ParseResult<CallArguments> {
        let tokg = self.require(
            TokenKind::LeftParen,
            TokenConsumptionKind::Absolute,
            &[],
            TokenMatchExpectation::Exact,
        )?;
        let span = tokg.span();

        let TokenGraph::Collection { data, .. } = tokg else {
            unreachable!()
        };

        let n = data.len();

        self.use_stream(
            TokenStream::new(data.into_iter().skip(1).take(n - 2).collect()),
            |s| -> ParseResult<CallArguments> {
                let mut args = CallArguments {
                    data: Vec::new(),
                    span,
                };
                let mut expect = true;

                while !s.eos() {
                    if !expect {
                        s.require_exact_nonlf(TokenKind::Comma)?;
                    }

                    if expect {
                        args.data.push(s.parse_expr(0)?);
                        expect = false;
                    }

                    if !expect && s.expect_exact_nonlf(TokenKind::Comma).0 {
                        expect = true;
                        continue;
                    }
                }

                Ok(args)
            },
        )
    }

    pub fn parse_ref_expr(&mut self) -> ParseResult<RefExpr> {
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

        Ok(RefExpr {
            span: span.merge(expr.span),
            expr: Box::new(expr),
            mutability,
        })
    }

    pub fn parse_array_expr(&mut self) -> ParseResult<ArrayExpr> {
        let Some(tokg) = self.next_nonlf() else {
            unreachable!()
        };

        let span = tokg.span();
        let TokenGraph::Collection { data, .. } = tokg else {
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

    pub fn parse_if_expr(&mut self) -> ParseResult<IfExpr> {
        let Some(tokg) = self.next_nonlf_token() else {
            unreachable!()
        };

        let cond = Box::new(self.use_ctx(ParserCtx::IfCond, |s| s.parse_expr(0))?);
        let consequent = Box::new(self.parse_if_expr_branch()?);

        let alternate = if let (true, Some(_)) =
            self.expect_exact_nonlf(TokenKind::Ident(TokenIdentKind::Else))
        {
            if let (true, Some(_)) =
                self.expect_preserved_exact_nonlf(TokenKind::Ident(TokenIdentKind::If))
            {
                let ite = self.parse_expr(0)?;
                Some(Block {
                    span: ite.span,
                    stmts: vec![Stmt::new(StmtKind::Pass(Box::new(PassStmt {
                        span: ite.span,
                        value: Some(Box::new(ite)),
                    })))],
                })
            } else {
                Some(self.parse_if_expr_branch()?)
            }
        } else {
            None
        };

        Ok(IfExpr {
            span: tokg.span.merge(
                alternate
                    .as_ref()
                    .map(|alt| alt.span)
                    .unwrap_or(consequent.span),
            ),
            cond,
            consequent,
            alternate: alternate.map(|alt| Box::new(alt)),
        })
    }

    pub fn parse_if_expr_branch(&mut self) -> ParseResult<Block> {
        ternary!(
            self.expect_preserved_exact_nonlf(TokenKind::LeftBrace).0,
            self.parse_block(),
            {
                self.require_exact_nonlf(TokenKind::Colon)?;

                let expr = self.parse_expr(0)?;
                let span = expr.span;

                Ok(Block {
                    stmts: vec![Stmt::new(StmtKind::Pass(Box::new(PassStmt {
                        value: Some(Box::new(expr)),
                        span,
                    })))],
                    span,
                })
            }
        )
    }

    // PATH { (FIELD (, FIELD)?)* }
    // PATH { (IDENT : EXPR (, IDENT : EXPR)?)* }
    pub fn parse_struct_expr(&mut self, path: Path) -> ParseResult<StructExpr> {
        let tokg = self.next_nonlf().unwrap();
        let span = path.span.merge(tokg.span());
        let TokenGraph::Collection { data, .. } = tokg else {
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

    // IDENT
    // IDENT : EXPR
    pub fn parse_struct_expr_field(&mut self) -> ParseResult<StructExprField> {
        // IDENT
        let ident = self.parse_raw_ident()?;

        // :
        if !self.expect_exact_nonlf(TokenKind::Colon).0 {
            return Ok(StructExprField {
                val: Box::new(Expr::new(ExprKind::Path(Box::new(Path::new(vec![
                    Identifier {
                        ident: ident.clone(),
                        span: ident.span,
                        arguments: None,
                    },
                ]))))),
                ident,
            });
        }

        // EXPR
        let val = Box::new(self.parse_expr(0)?);

        Ok(StructExprField { ident, val })
    }

    // fn ( (PARAM (, PARAM)*)? ) (-> RET_TY)? BODY
    // fn ( (PARAM (, PARAM)*)? ) (-> RET_TY)? (BLOCK | EXPR)
    pub fn parse_anon_fn(&mut self, span: Span) -> ParseResult<AnonFn> {
        // ( (PARAM (, PARAM)*)? )
        let params = self.parse_anon_fn_param_list()?;

        // ->
        let mut ret_ty: Option<Box<Ty>> = None;
        if self.expect_exact_nonlf(TokenKind::MinusGreater).0 {
            // RET_TY
            ret_ty = Some(Box::new(self.parse_ty()?));
        }

        // BODY
        let body = ternary!(
            self.expect_preserved_exact_nonlf(TokenKind::LeftBrace).0,
            self.parse_block()?,
            {
                let expr = self.parse_expr(0)?;
                let span = expr.span;

                Block {
                    stmts: vec![Stmt::new(StmtKind::Ret(Box::new(RetStmt {
                        value: Some(Box::new(expr)),
                        span,
                    })))],
                    span,
                }
            }
        );

        Ok(AnonFn {
            span: span.merge(body.span),
            params,
            ret_ty,
            body,
        })
    }

    pub fn parse_anon_fn_param_list(&mut self) -> ParseResult<AnonFnParamList> {
        let Ok(tokg) = self.require_exact_nonlf(TokenKind::LeftParen) else {
            return Err(None);
        };

        let span = tokg.span();
        let TokenGraph::Collection { data, .. } = tokg else {
            unreachable!()
        };

        let n = data.len();
        self.use_stream(
            TokenStream::new(data.into_iter().skip(1).take(n - 2).collect()),
            |s| {
                let mut params = AnonFnParamList {
                    list: Vec::new(),
                    span,
                };

                let mut expect = true;
                while !s.eos() {
                    if !expect {
                        s.require_exact_nonlf(TokenKind::Comma)?;
                    }

                    if expect {
                        params.list.push(s.parse_anon_fn_param()?);
                        expect = false;
                    }

                    if !expect && s.expect_exact_nonlf(TokenKind::Comma).0 {
                        expect = true;
                        continue;
                    }
                }

                Ok(params)
            },
        )
    }

    pub fn parse_anon_fn_param(&mut self) -> ParseResult<AnonFnParam> {
        // IDENT
        let ident = self.parse_raw_ident()?;

        // Optional type annotation
        let mut ty: Option<Box<Ty>> = None;
        if self.expect_exact_nonlf(TokenKind::Colon).0 {
            ty = Some(Box::new(self.parse_ty()?));
        }

        Ok(AnonFnParam { ident, ty })
    }

    pub fn parse_fn_call(&mut self, left: Expr) -> ParseResult<FnCall, (Expr, Option<ParserDiag>)> {
        let arguments = match self.parse_fn_call_arguments() {
            Ok(arguments) => arguments,
            Err(diag) => return Err((left, diag)),
        };

        Ok(FnCall {
            callee: Box::new(left),
            arguments,
        })
    }

    pub fn parse_field_access_or_method_call(
        &mut self,
        left: Expr,
    ) -> ParseResult<ExprKind, (Expr, Option<ParserDiag>)> {
        self.adjust_to_nonlf();

        // Field access for tuples or any integer fields
        if let (true, Some(tokg)) = self.expect_similar_nonlf(TokenKind::Int { base: 64 }) {
            return Ok(ExprKind::FieldAccess(Box::new(FieldAccess {
                leading: Box::new(left),
                field: tokg.underlying().unwrap().clone(),
            })));
        }

        let ident = match self.parse_ident(PathKind::Expr) {
            Ok(ident) => ident,
            Err(diag) => return Err((left, diag)),
        };

        // If the identifier consists of arguments, it is guaranteed
        // to be a method call.
        let kind = if ident.arguments.is_some() {
            let arguments = match self.parse_fn_call_arguments() {
                Ok(arguments) => arguments,
                Err(diag) => return Err((left, diag)),
            };

            ExprKind::MethodCall(Box::new(MethodCall {
                receiver: Box::new(left),
                callee: ident,
                arguments,
            }))
        } else {
            // If the next token is a `LeftParen`, it is a method call
            if self.expect_preserved_exact_nonlf(TokenKind::LeftParen).0 {
                let arguments = match self.parse_fn_call_arguments() {
                    Ok(arguments) => arguments,
                    Err(diag) => return Err((left, diag)),
                };

                ExprKind::MethodCall(Box::new(MethodCall {
                    receiver: Box::new(left),
                    callee: ident,
                    arguments,
                }))
            } else {
                ExprKind::FieldAccess(Box::new(FieldAccess {
                    leading: Box::new(left),
                    field: ident.ident,
                }))
            }
        };

        Ok(kind)
    }
}
