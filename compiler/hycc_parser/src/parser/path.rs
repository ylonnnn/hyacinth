use hycc_ast::{
    Identifier, Path,
    path::{IdentifierArgument, IdentifierArguments},
    token::{Token, TokenIdentKind, TokenKind},
};

use crate::parser::{Parser, parser::ParseResult};

impl<'d, 's> Parser<'d, 's> {
    pub fn parse_raw_ident(&mut self) -> ParseResult<Token> {
        let Some(tg) = self.require_abs_similar_nonlf(TokenKind::Ident(TokenIdentKind::Normal))
        else {
            return Err(true);
        };

        let Some(tok) = tg.underlying() else {
            return Err(false);
        };

        Ok(tok.clone())
    }

    // IDENT (:: IDENT)*
    pub fn parse_path(&mut self) -> ParseResult<Path> {
        let lead = self.parse_ident()?;
        let mut path = Path::new(vec![lead]);

        while self.expect_exact_nonlf(TokenKind::ColonColon).0 {
            path.add(self.parse_ident()?)
        }

        Ok(path)
    }

    // RAW_IDENT < GENERIC_ARG (, GENERIC_ARG)* >
    pub fn parse_ident(&mut self) -> ParseResult<Identifier> {
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
            args = self.parse_ident_args();

            // >
            if self.expect_exact_nonlf(TokenKind::Greater).0 {
            }
            // If the generic argument closing token did not match, it may
            // be a bound token (e.g. >>, >=)
            else {
                if self
                    .expect_preserved_exact_nonlf(TokenKind::GreaterGreater)
                    .0
                {
                    // Requires N "<" encounters for the current token to be consumed
                    self.generic_delimeter_encounters += 1;
                    if self.generic_delimeter_encounters % 2 == 0 {
                        self.adjust_to_nonlf();
                    }

                    closed = true;
                }

                // TODO: for the token kind ">="
            }
        }

        if !closed {
            // If the argument count of is less than or equal to one (1),
            // The expression may have been misinterpreted as a path rather than
            // a binary expression such as a < b
            if let Some(args) = &args
                && args.data.len() <= 1
            {
                // TODO
                // Misdiagnose the most recent error if it exists

                // Revert to the initial position/offset

                return Ok(Identifier::new(raw_ident?, None));
            }

            // If not misinterpreted, simply emit an error
            // requiring a closing delimeter
            self.require_exact_nonlf(TokenKind::Greater);
        }

        Ok(Identifier::new(raw_ident?, args))
    }

    // < GENERIC_ARG (, GENERIC_ARG) >
    pub fn parse_ident_args(&mut self) -> Option<IdentifierArguments> {
        let tg = self.next_nonlf()?;
        let op_delim = tg.underlying()?;

        let mut arguments = IdentifierArguments {
            data: Vec::new(),
            span: op_delim.span,
        };
        let mut expect = true;

        while !self.expect_preserved_exact_nonlf(TokenKind::Greater).0
            && !self
                .expect_preserved_exact_nonlf(TokenKind::GreaterGreater)
                .0
        {
            if expect {
                if let Some(arg) = self.parse_ident_arg() {
                    arguments.data.push(arg);
                }

                expect = false;
            }

            if !expect && self.expect_exact_nonlf(TokenKind::Comma).0 {
                expect = true;
                continue;
            }

            break;
        }

        Some(arguments)
    }

    // GENERIC_ARG ::= { EXPR } | TYPE
    pub fn parse_ident_arg(&mut self) -> Option<IdentifierArgument> {
        if self.expect_exact_nonlf(TokenKind::LeftBrace).0 {
            // TODO: parse expression
            None
        } else {
            // TODO: parse type
            None
        }
    }
}
