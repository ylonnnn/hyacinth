use crate::core::{Position, Program};

#[derive(Debug, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn extend(&self, offset: usize) -> Self {
        let (start, end) = self.clone().into();
        Self {
            start,
            end: end + offset,
        }
    }

    pub fn to_rc(&self, program: &Program) -> (Position, Position) {
        let source = &program.lexer.source;
        let Span { start, end } = &self;

        let (mut s_pos, mut e_pos) = (Position::default(), Position::default());
        let (mut line, mut column) = (1_usize, 1_usize);

        source.char_indices().for_each(|(i, c)| {
            for (offset, pos) in [(start, &mut s_pos), (end, &mut e_pos)] {
                if *offset != i {
                    continue;
                }

                (pos.line, pos.column) = (line, column);
            }

            column += 1;

            if c == '\n' {
                (line, column) = (line + 1, 1);
            }
        });

        (s_pos, e_pos)
    }
}

impl From<(usize, usize)> for Span {
    fn from(value: (usize, usize)) -> Self {
        let (start, end) = value;
        Self { start, end }
    }
}

impl Into<(usize, usize)> for Span {
    fn into(self) -> (usize, usize) {
        (self.start, self.end)
    }
}
