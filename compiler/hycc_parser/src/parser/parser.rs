use crate::lexer::Lexer;

#[derive(Debug)]
pub struct Parser<'l, 's> {
    lexer: Lexer<'l, 's>,
}

impl<'l, 's> Parser<'l, 's> {
    pub fn new(lexer: Lexer<'l, 's>) -> Self {
        Self { lexer }
    }

    pub fn parse(&mut self) {
    }
}
