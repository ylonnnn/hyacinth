#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn extend(&self, end: usize) -> Self {
        let (start, c_end) = self.clone().into();
        Self {
            start,
            end: c_end + end,
        }
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
