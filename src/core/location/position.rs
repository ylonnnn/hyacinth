use std::fmt::Display;

#[derive(Debug, Clone, Copy, Default)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}
