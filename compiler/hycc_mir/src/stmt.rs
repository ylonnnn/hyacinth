use std::fmt::Display;

use hycc_const::table::ConstId;
use hycc_hir::expr;
use hycc_span::Span;

use crate::local::LocalDeclId;

#[derive(Debug, Clone)]
pub enum MirStatementKind {
    Nop,

    Assign(Box<(Location, RValue)>),
    StorageLive(LocalDeclId),
    StorageDead(LocalDeclId),
    // Deinit(Location),
    // SetDiscriminant(Location, VariantIdx),
}

#[derive(Debug, Clone)]
pub struct MirStatement {
    pub kind: MirStatementKind,
    pub span: Span,
}

impl MirStatement {
    pub fn new(kind: MirStatementKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl Display for MirStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            MirStatementKind::Assign(assign) => {
                write!(f, "_{} = {}", assign.0.decl.0, assign.1)
            }

            _ => write!(f, "{:?}", self.kind),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Location {
    pub decl: LocalDeclId,
    pub projection: Vec<Projection>,
}

impl Location {
    pub fn new(decl: LocalDeclId) -> Self {
        Self {
            decl,
            projection: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Projection {
    Deref,
    // Field(FieldIdx, Ty),
    // Index(LocalId),
    // ConstantIndex {
    //     offset: u64,
    //     min_length: u64,
    //     from_end: bool,
    // },
    // Downcast(Option<Symbol>, VariantIdx),
}

pub type BinaryOp = expr::BinaryOp;

#[derive(Debug, Clone)]
pub enum RValue {
    Use(Operand),
    Ref(RefKind, Location),
    // Len(Location),
    // Cast(CastKind, Operand, Ty),
    Binary(BinaryOp, Box<(Operand, Operand)>),
    // UnaryOp(UnOp, Operand),
    // NullaryOp(NullOp, Ty),
    Discriminant(Location),
    // Aggregate(Box<AggregateKind>, Vec<Operand>),
}

impl Display for RValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Use(op) => write!(f, "{}", &op),

            _ => write!(f, "{:?}", &self),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RefKind {
    Mutable,
    Immutable,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Copy(Location),
    Move(Location),
    Const(ConstId),
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Copy(loc) => write!(f, "copy _{}", loc.decl.0),
            Self::Move(loc) => write!(f, "move _{}", loc.decl.0),
            Self::Const(id) => write!(f, "const {:?}", id),
        }
    }
}
