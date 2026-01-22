use std::{collections::HashMap, fmt::Debug};

use crate::{
    coalesce,
    core::diagnostic,
    syntax::{
        Node, Parser, ParserState, TokenKind,
        grammar::rule::GrammarRule,
        rule::{
            GrammarContext,
            common::{CommonRules, PrattRule},
        },
    },
};

#[derive(Debug)]
pub struct Grammar {
    rules: HashMap<TokenKind, Box<dyn GrammarRule>>,
    fallback: Box<dyn GrammarRule>,
}

impl Grammar {
    pub fn new() -> Self {
        Self {
            rules: HashMap::with_capacity(8),
            fallback: Box::new(PrattRule::new()),
        }
    }

    pub fn add(&mut self, leader: TokenKind, rule: Box<dyn GrammarRule>) {
        self.rules.insert(leader, rule);
    }

    pub fn parse(&mut self, parser: &mut Parser, context: GrammarContext) -> Option<Node> {
        let lexer = &mut parser.program.lexer;
        if lexer.abs_eof() {
            return None;
        }

        let token = lexer.peek()?.clone();
        let rule = coalesce!(self.rules.get_mut(&token.kind), &mut self.fallback);

        if !rule.context().contains(context) {
            lexer.consume(1);

            let diagnostic =
                diagnostic::syntax_error_unexpected_token(&parser.program.lexer.source, &token);
            parser.program.diagnostic_list_mut().add(diagnostic);

            return None;
        }

        let data = rule.parse(parser);
        if parser.is(ParserState::Panic) {
            rule.recover(parser);
        }

        data
    }
}
