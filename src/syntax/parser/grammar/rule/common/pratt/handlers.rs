use crate::syntax::{Expr, GenNode, LiteralExpr, NodeData, Parser, TokenKind};

pub fn parse_literal(parser: &mut Parser) -> Option<GenNode<NodeData>> {
    let lexer = &mut parser.program.lexer;
    let token = lexer.next()?;

    let sv = token.view(&parser.program.lexer.source);

    let expr = match token.kind {
        TokenKind::Int => match sv.parse::<i64>() {
            Ok(x) => Some(LiteralExpr::Int(x)),
            _ => None,
        },

        TokenKind::Float => match sv.parse::<f64>() {
            Ok(x) => Some(LiteralExpr::Float(x)),
            _ => None,
        },

        TokenKind::Bool => match sv.parse::<bool>() {
            Ok(x) => Some(LiteralExpr::Bool(x)),
            _ => None,
        },

        // TODO: Add other literal expression types
        _ => None,
    }?;

    Some(GenNode::new(Expr::Literal(expr).into(), token.span))
}
