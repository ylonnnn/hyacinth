use crate::core::{Position, source::ProgramSource};

#[derive(Debug, Clone)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn extend(&self, offsets: (u32, u32)) -> Self {
        let (start, end) = self.clone().into();
        let (s_offset, e_offset) = offsets;

        Self {
            start: start + s_offset,
            end: end + e_offset,
        }
    }

    pub fn extend_start(mut self, s_offset: u32) -> Self {
        self.start += s_offset;
        self
    }

    pub fn extend_end(mut self, e_offset: u32) -> Self {
        self.end += e_offset;
        self
    }

    pub fn to_position_range(&self, source: &ProgramSource) -> (Position, Position) {
        let Span { start, end } = &self;
        let convert = |mut offset: u32| -> Position {
            for (line, i) in source.lines.iter().zip(0_u32..) {
                let len = line.len() as u32;
                if offset <= len {
                    return Position {
                        line: i + 1,
                        column: offset + 1,
                    };
                }

                offset -= len + 1
            }

            unreachable!()
        };

        (convert(*start), convert(*end))
    }
}

impl Default for Span {
    fn default() -> Self {
        Self { start: 0, end: 0 }
    }
}

impl From<(u32, u32)> for Span {
    fn from(value: (u32, u32)) -> Self {
        let (start, end) = value;
        Self { start, end }
    }
}

impl Into<(u32, u32)> for Span {
    fn into(self) -> (u32, u32) {
        (self.start, self.end)
    }
}
