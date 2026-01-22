use crate::syntax::{
    GenNode, Item, Node, Parser, Stmt, TERMINATOR, TokenKind, VariableDeclStmt,
    rule::{GrammarContext, GrammarRule, common::PrattHandlerKind},
};

#[derive(Debug)]
pub struct VariableRule {
    leader: TokenKind,
    context: GrammarContext,
}

impl VariableRule {
    pub fn new() -> Self {
        Self {
            leader: TokenKind::Let,
            context: GrammarContext::LOCAL.with(GrammarContext::GLOBAL),
        }
    }
}

impl GrammarRule for VariableRule {
    fn leader(&self) -> TokenKind {
        self.leader.clone()
    }

    fn context(&self) -> GrammarContext {
        self.context.clone()
    }

    fn parse(&mut self, parser: &mut Parser) -> Option<Node> {
        // "let" "constexpr"? "mut"? IDENTIFIER ( ":" TYPE )? ( "=" VALUE )? ;
        // let IDENTIFIER (: TYPE)? = VALUE

        let start = parser.require_nonlf(TokenKind::Let)?.span.start;
        let ident = parser.require_nonlf(TokenKind::Ident)?;

        // TODO: (: TYPE)?

        parser.require_nonlf(TokenKind::Eq)?;

        let mut pratt = parser.common.pratt.take().unwrap();
        let value = pratt.parse_base(parser, 0_f32, PrattHandlerKind::Expr);
        parser.common.pratt.replace(pratt);

        let end = parser.require_then_consume(TERMINATOR)?.span.end;

        Some(Node::from_item(
            Item::Variable(VariableDeclStmt {
                ident,
                value: value.map(|mut val| val.expr_node()).unwrap(),
            }),
            (start, end).into(),
        ))
    }

    fn recover(&self, parser: &mut Parser) {
        parser.sync_with(vec![TokenKind::LnFeed]);
    }
}
