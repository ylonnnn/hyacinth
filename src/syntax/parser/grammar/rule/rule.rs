use std::fmt::Debug;

use crate::syntax::{Node, TokenKind, parser::parser::Parser};

pub trait GrammarRule
where
    Self: Debug,
{
    fn leader(&self) -> TokenKind {
        TokenKind::Eof
    }

    fn context(&self) -> GrammarContext;

    fn parse(&mut self, parser: &mut Parser) -> Option<Node>;
    fn recover(&self, parser: &mut Parser);
}

#[derive(Debug, Clone)]
pub struct GrammarContext(u8);

impl GrammarContext {
    pub const NONE: Self = Self(0);
    pub const LOCAL: Self = Self(1 << 0);
    pub const GLOBAL: Self = Self(1 << 1);

    pub fn with(&self, context: GrammarContext) -> Self {
        GrammarContext(self.0 | context.0)
    }

    pub fn contains(&self, context: GrammarContext) -> bool {
        (self.0 & context.0) == context.0
    }
}
