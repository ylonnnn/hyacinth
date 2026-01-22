use std::{collections::HashMap, fmt::Debug};

use crate::{
    core::diagnostic,
    hashmap,
    syntax::{
        Node, Parser, TokenKind,
        grammar::rule::GrammarRule,
        rule::{GrammarContext, common::parse_literal},
    },
};

#[derive(Debug)]
pub struct PrattRule {
    context: GrammarContext,
    handlers: HashMap<PrattHandlerKind, HashMap<TokenKind, PrattHandler>>,
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrattHandlerKind {
    Expr,
    Type,
}

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum ExprBindingPower {
    Default,
    Comma,
    Assignment,
    Statement,
    ConditionalSelection,
    ConditionalLogical,
    Bitwise,
    Relational,
    BitwiseShift,
    Additive,
    Multiplicative,
    Exponentiation,
    Unary,
    FunctionCall,
    MemberAccess,
    Primary,
}

impl From<ExprBindingPower> for f32 {
    fn from(value: ExprBindingPower) -> Self {
        value as u8 as f32
    }
}

pub struct PrattHandler {
    pub kind: TokenKind,
    pub binding_power: (f32, f32),
    pub nud: Option<Box<dyn FnMut(&mut Parser) -> Option<Node>>>,
    pub led: Option<Box<dyn FnMut(&mut Parser, Option<&mut Node>, f32) -> Option<Node>>>,
}

impl PrattRule {
    pub fn new() -> Self {
        let mut inst = Self {
            context: GrammarContext::LOCAL,
            handlers: hashmap! {
                PrattHandlerKind::Expr => HashMap::new(),
                PrattHandlerKind::Type => HashMap::new(),
            },
        };

        inst.initialize();

        inst
    }

    pub fn initialize(&mut self) {
        if let Some(handlers) = self.handlers.get_mut(&PrattHandlerKind::Expr) {
            let primary_bp: f32 = ExprBindingPower::Primary.into();
            vec![TokenKind::Int, TokenKind::Float, TokenKind::Bool]
                .into_iter()
                .for_each(|kind| {
                    handlers.insert(
                        kind.clone(),
                        PrattHandler {
                            kind,
                            binding_power: (primary_bp, primary_bp),
                            nud: Some(Box::new(parse_literal)),
                            led: None,
                        },
                    );
                });
        }
    }

    pub fn get_handler(
        &mut self,
        handler_kind: &PrattHandlerKind,
        kind: &TokenKind,
    ) -> Option<&mut PrattHandler> {
        self.handlers.get_mut(&handler_kind)?.get_mut(&kind)
    }

    pub fn parse_base(
        &mut self,
        parser: &mut Parser,
        right_bp: f32,
        kind: PrattHandlerKind,
    ) -> Option<Node> {
        parser.program.lexer.skip_lf();

        let lexer = &parser.program.lexer;
        let token = lexer.peek()?;

        let handler = self.get_handler(&kind, &token.kind)?;

        let Some(nud) = &mut handler.nud else {
            diagnostic::helper::syntax_error_unexpected_token(&parser.program.lexer.source, token);
            return None;
        };

        let mut left: Option<Node> = nud.as_mut()(parser);
        while !parser.program.lexer.eof(false) {
            let Some(token) = parser.program.lexer.peek() else {
                break;
            };
            let Some(handler) = self.get_handler(&kind, &token.kind) else {
                break;
            };

            let (bp, led) = (handler.binding_power, &mut handler.led);
            let (r_bp, l_bp) = bp;

            if right_bp > l_bp || led.is_none() {
                break;
            }

            if let Some(led) = led.as_mut() {
                led.as_mut()(parser, left.as_mut(), r_bp);
            }
        }

        left
    }
}

impl GrammarRule for PrattRule {
    fn context(&self) -> GrammarContext {
        self.context.clone()
    }

    fn parse(&mut self, parser: &mut Parser) -> Option<Node> {
        self.parse_base(parser, 0_f32, PrattHandlerKind::Expr)
    }

    fn recover(&self, parser: &mut Parser) {
        parser.sync();
    }
}

impl Debug for PrattHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrattHandler")
            .field("nud", &"<dyn FnMut(&mut Parser)>")
            .field("led", &"<dyn FnMut(&mut Parser, Option<&Node>, f32)>")
            .finish()
    }
}
