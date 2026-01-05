use std::collections::HashSet;

use crate::{
    core::{Program, diagnostic::code::DiagnosticErrorKind},
    syntax::{Token, TokenConsumptionType, TokenKind},
};

#[derive(Debug)]
pub struct Parser<'a> {
    pub program: &'a mut Program,
    pub state: ParserState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserState {
    Panic,
    Synchronized,
}

impl<'a> Parser<'a> {
    pub fn new(program: &'a mut Program) -> Self {
        Self {
            program,
            state: ParserState::Synchronized,
        }
    }

    pub fn is(&self, state: ParserState) -> bool {
        self.state == state
    }

    pub fn panic(&mut self) {
        self.state = ParserState::Panic
    }

    pub fn sync(&mut self) {
        self.state = ParserState::Synchronized
    }

    pub fn sync_with(&mut self, tokens: Vec<TokenKind>) {
        #[allow(unused)]
        let mut set: HashSet<TokenKind> = tokens.into_iter().collect();
    }

    pub fn expect(&mut self, kind: TokenKind, consumption: TokenConsumptionType) -> Option<Token> {
        self.program.lexer.expect(kind, consumption)
    }

    pub fn expect_or_error(&mut self, kind: TokenKind, consumption: TokenConsumptionType) {
        if let Some(token) = self.expect(kind, consumption) {
            self.program.diagnostic_list_mut().error(
                DiagnosticErrorKind::UnexpectedToken.into(),
                &format!("unexpected token `{}`", token),
                token.span.clone(),
            );
        }
    }
}
