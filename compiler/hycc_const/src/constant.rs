use std::rc::Rc;

use hycc_span::Span;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstKind {
    Int(u64),
    Float(u64),
    Bool(bool),
    Char(u8),
    String(Rc<str>),
}

impl ConstKind {
    pub fn float(data: f64) -> Self {
        Self::Float(data.to_bits())
    }

    pub fn as_float(&self) -> f64 {
        let Self::Float(data) = &self else {
            panic!("const kind is not a float!")
        };

        f64::from_bits(*data)
    }
}

#[derive(Debug, Clone)]
pub struct Const {
    pub kind: ConstKind,
    pub span: Span,
}
