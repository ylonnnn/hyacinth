use std::collections::HashSet;

use crate::{
    core::{
        Program,
        diagnostic::{self},
    },
    syntax::{
        GenNode, Grammar, Item, ProgramNode, Token, TokenConsumptionType, TokenKind,
        rule::{GrammarContext, common::CommonRules, items::hyacinth},
    },
};

pub const TERMINATOR: TokenKind = TokenKind::LnFeed;

#[derive(Debug)]
pub struct Parser<'a> {
    pub program: &'a mut Program,
    pub state: ParserState,
    pub grammar: Option<Grammar>,
    pub common: CommonRules,
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
            grammar: Some(Grammar::new()),
            common: CommonRules::new(),
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
        let set: HashSet<TokenKind> = tokens.into_iter().collect();

        while let Some(token) = self.program.lexer.next() {
            if !set.contains(&token.kind) {
                continue;
            }

            self.sync();
        }
    }

    pub fn expect(
        &mut self,
        kind: TokenKind,
        consumption: TokenConsumptionType,
        exclude: Vec<TokenKind>,
    ) -> Option<Token> {
        self.program.lexer.expect(kind, consumption, exclude)
    }

    pub fn require_with_exclusion(
        &mut self,
        kind: TokenKind,
        consumption: TokenConsumptionType,
        exclude: Vec<TokenKind>,
    ) -> Option<Token> {
        if let Some(token) = self.expect(kind.clone(), consumption, exclude) {
            Some(token)
        } else {
            let program = &mut self.program;
            let token = program.lexer.peek()?;

            let diagnostic = diagnostic::helper::syntax_error_expectation_mismatch(
                &program.lexer.source,
                token,
                Some(kind),
            );

            program.diagnostic_list_mut().add(diagnostic);

            None
        }
    }

    pub fn require(&mut self, kind: TokenKind, consumption: TokenConsumptionType) -> Option<Token> {
        self.require_with_exclusion(kind, consumption, vec![])
    }

    pub fn require_wexc_then_consume(
        &mut self,
        kind: TokenKind,
        exclude: Vec<TokenKind>,
    ) -> Option<Token> {
        self.require_with_exclusion(kind, TokenConsumptionType::UponSuccess, exclude)
    }

    pub fn require_then_consume(&mut self, kind: TokenKind) -> Option<Token> {
        self.require_wexc_then_consume(kind, vec![])
    }

    /**
     * NOTE: Requires the next token to be the provided kind and
     * excludes line feeds. Consumes tokens upon success.
     */
    pub fn require_nonlf(&mut self, kind: TokenKind) -> Option<Token> {
        self.require_wexc_then_consume(kind, vec![TokenKind::LnFeed])
    }

    pub fn parse(&mut self) -> Option<GenNode<ProgramNode>> {
        let mut grammar = self.grammar.take().unwrap_or_else(Grammar::new);
        hyacinth::initialize(&mut grammar);

        let mut items = Vec::<GenNode<Item>>::with_capacity(16);

        while !self.program.lexer.eof(false) {
            let Some(mut data) = grammar.parse(self, GrammarContext::GLOBAL) else {
                break;
            };

            if let Some(item) = data.item_node() {
                items.push(item);
            }
        }

        self.grammar = Some(grammar);
        let end = self.program.lexer.source.len();

        Some(GenNode::new(
            ProgramNode::new(self.program, items),
            (0, end).into(),
        ))
    }
}
