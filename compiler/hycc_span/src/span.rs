use crate::Position;

use hycc_source::source::Source;

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub offset: u32,
    pub len: u16,
    pub src_id: u16,
}

impl Span {
    pub fn new(offset: u32, len: u16, program_id: u16) -> Self {
        Self {
            offset,
            len,
            src_id: program_id,
        }
    }

    pub fn merge(&self, other: &Span) -> Span {
        assert!(
            self.src_id == other.src_id,
            "both spans must have the same sources for them to be merged!"
        );

        let start = self.offset.min(other.offset);
        Self::new(
            start,
            (self.end().max(other.end()) - start) as u16,
            self.src_id,
        )
    }

    #[inline]
    pub const fn end(&self) -> u32 {
        self.offset + self.len as u32
    }

    pub fn extend(mut self, n: u16) -> Self {
        self.len += n;
        self
    }

    pub fn to_position_range(&self, source: &Source) -> (Position, Position) {
        let Span { offset, len, .. } = &self;
        let convert = |mut offset: u32| -> Position {
            for (line, i) in source.data.lines().zip(0_u32..) {
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

        (convert(*offset), convert(*offset + (*len as u32)))
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::new(0, 0, u16::MAX)
    }
}

impl From<(u32, u16, u16)> for Span {
    fn from(value: (u32, u16, u16)) -> Self {
        let (offset, len, src_id) = value;
        Self {
            offset,
            len,
            src_id,
        }
    }
}
