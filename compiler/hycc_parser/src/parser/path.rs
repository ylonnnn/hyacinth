use hycc_ast::{
    path::{Identifier, Path},
    path::{IdentifierArgument, IdentifierArguments},
    token::{Token, TokenGraph, TokenIdentKind, TokenKind},
    token_stream::{TokenConsumptionKind, TokenMatchExpectation, TokenStream},
};
use hycc_diagnostic::DiagnosticContext;

use crate::parser::{Parser, parser::ParseResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathKind {
    None,
    Expr,
    Ty,
}

impl<'s> Parser<'s> {
    pub fn parse_raw_ident(&mut self) -> ParseResult<Token> {
        let tokg = self.require_abs_similar_nonlf(TokenKind::Ident(TokenIdentKind::Normal))?;
        let Some(tok) = tokg.underlying() else {
            return Err(None);
        };

        Ok(tok.clone())
    }

    // IDENT (:: IDENT)*
    pub fn parse_path(&mut self, kind: PathKind) -> ParseResult<Path> {
        let lead = self.parse_ident(kind)?;
        let mut path = Path::new(vec![lead]);

        while self.expect_exact_nonlf(TokenKind::ColonColon).0 {
            path.add(self.parse_ident(kind)?)
        }

        Ok(path)
    }

    // RAW_IDENT < GENERIC_ARG (, GENERIC_ARG)* >
    pub fn parse_ident(&mut self, kind: PathKind) -> ParseResult<Identifier> {
        // RAW_IDENT
        let raw_ident = self.parse_raw_ident();

        // <
        let mut closed = true;
        let mut args = Option::<IdentifierArguments>::None;
        // auto initial = context.offset();

        if let (matched, Some(_)) = self.expect_preserved_exact_nonlf(TokenKind::Less)
            && matched
        {
            closed = false;
            match self.parse_ident_args() {
                Ok(arguments) => args = Some(arguments),
                Err(diag) => {
                    if let Some(diag) = diag {
                        self.dctx.add(diag);
                    }
                }
            }

            // >
            match self.require(
                TokenKind::Greater,
                TokenConsumptionKind::UponSuccess,
                &[],
                TokenMatchExpectation::Exact,
            ) {
                Ok(_) => closed = true,
                Err(diag) => {
                    if let Some(diag) = diag {
                        self.dctx.add(diag);
                    }
                }
            }
        }

        if !closed {
            // If the argument count of is less than or equal to one (1),
            // The expression may have been misinterpreted as a path rather than
            // a binary expression such as a < b
            if let Some(args) = &args
                && args.data.len() <= 1
                && kind == PathKind::Expr
            {
                // TODO
                // Misdiagnose the most recent error if it exists

                // Revert to the initial position/offset

                return Ok(Identifier::new(raw_ident?, None));
            }

            // If not misinterpreted, simply emit an error
            // requiring a closing delimeter
            self.require_exact_nonlf(TokenKind::Greater)?;
        }

        Ok(Identifier::new(raw_ident?, args))
    }

    // < GENERIC_ARG (, GENERIC_ARG) >
    pub fn parse_ident_args(&mut self) -> ParseResult<IdentifierArguments> {
        let Some(op_delim) = self.next_nonlf_token() else {
            return Err(None);
        };

        let mut data = Vec::new();
        let mut expect = true;

        while !self.expect_preserved_exact_nonlf(TokenKind::Greater).0 {
            if !expect {
                self.require_exact_nonlf(TokenKind::Comma)?;
            }

            if expect {
                data.push(self.parse_ident_arg()?);
                expect = false;
            }

            if !expect && self.expect_exact_nonlf(TokenKind::Comma).0 {
                expect = true;
                continue;
            }
        }

        Ok(IdentifierArguments {
            data,
            span: op_delim.span.merge(self.peek_nonlf_token().unwrap().span),
        })
    }

    // GENERIC_ARG ::= { EXPR } | TYPE
    pub fn parse_ident_arg(&mut self) -> ParseResult<IdentifierArgument> {
        if let (_, Some(TokenGraph::Collection { data, .. })) =
            self.expect_exact_nonlf(TokenKind::LeftBrace)
        {
            let n = data.len();
            self.use_stream(
                TokenStream::new(data.into_iter().skip(1).take(n - 2).collect()),
                |s| Ok(IdentifierArgument::Expr(Box::new(s.parse_expr(0)?))),
            )
        } else {
            Ok(IdentifierArgument::Ty(Box::new(self.parse_ty()?)))
        }
    }
}
