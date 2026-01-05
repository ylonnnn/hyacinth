use std::{any::Any, collections::HashSet};

use crate::{
    core::{Program, diagnostic::code::DiagnosticErrorKind},
    syntax::{Token, TokenKind},
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserConsumptionType {
    Absolute,
    Preserve,
    UponSucess,
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

    pub fn expect(
        &mut self,
        kind: TokenKind,
        consumption: ParserConsumptionType,
    ) -> Option<&Token> {
        // let token = match consumption {
        //     ParserConsumptionType::Absolute => self.program.lexer.next(),
        //     ParserConsumptionType::Preserve => self.program.lexer.peek(),
        //     ParserConsumptionType::UponSucess => self.program.lexer.expect(kind),
        // }?;

        // Some(token)
        todo!()
    }

    pub fn expect_or_error(&mut self, kind: TokenKind, consumption: ParserConsumptionType) {
        // let (token, span) = {
        //     let Some(token) = self.program.lexer.peek() else {
        //         return;
        //     };

        //     (token.to_string(), token.span.clone())
        // };

        // if !self.expect(kind, consumption) {
        //     self.program.diagnostic_list().error(
        //         DiagnosticErrorKind::UnexpectedToken.into(),
        //         &format!("unexpected token `{}`", token),
        //         span,
        //     );
        // }

        todo!()
    }
}
