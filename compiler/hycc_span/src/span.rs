use crate::Position;

use hycc_source::source::{Source, SourceId};

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub offset: u32,
    pub len: u16,
    pub src_id: SourceId,
}

impl Span {
    pub fn new(offset: u32, len: u16, src_id: SourceId) -> Self {
        Self {
            offset,
            len,
            src_id,
        }
    }

    pub fn dummy(src_id: SourceId) -> Self {
        Self::new(0, 1, src_id)
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
            let mut line_no = 0;
            for line in source.data.lines() {
                line_no += 1;
                let len = line.len() as u32;
                if offset <= len {
                    return Position {
                        line: line_no,
                        column: offset + 1,
                    };
                }

                offset -= len + 1
            }

            Position {
                line: line_no,
                column: offset + 2,
            }
        };

        (convert(*offset), convert(*offset + (*len as u32)))
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::new(0, 0, SourceId(u16::MAX))
    }
}

impl From<(u32, u16, u16)> for Span {
    fn from(value: (u32, u16, u16)) -> Self {
        let (offset, len, src_id) = value;
        Self::new(offset, len, SourceId(src_id))
    }
}

impl From<(u32, u16, SourceId)> for Span {
    fn from(value: (u32, u16, SourceId)) -> Self {
        let (offset, len, src_id) = value;
        Self {
            offset,
            len,
            src_id,
        }
    }
}
