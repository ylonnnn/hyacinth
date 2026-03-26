use hycc_ast::{Stmt, StmtKind};

use crate::parser::Parser;

impl<'d, 's> Parser<'d, 's> {
    pub fn parse_stmt(&mut self) -> Option<Stmt> {
        let tg = self.next_nonlf()?;
        let Some(tok) = tg.underlying() else {
            return None;
        };

        match tok.kind {
            // TODO: implement other statements
            // TokenKind::Ident(..) => None,
            _ => {
                if let Some(item) = self.parse_item() {
                    Some(Stmt::new(StmtKind::Item(item)))
                } else {
                    println!("parse expressions");
                    None
                }
            }
        }
    }
}
