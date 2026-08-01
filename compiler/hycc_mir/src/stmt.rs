use std::fmt::Display;

use hycc_const::table::ConstId;
use hycc_hir::{def::DefId, expr};
use hycc_span::Span;
use hycc_ty::context::TyId;
use hycc_util::ternary;

use crate::{
    body::MirBodyId,
    decl::{GlobalDeclId, LocalDeclId},
};

#[derive(Debug, Clone)]
pub enum MirStatementKind {
    Nop,

    Assign(Box<(Place, RValue)>),
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
                write!(f, "{} = {}", assign.0, assign.1)
            }

            MirStatementKind::StorageLive(local_id) => write!(f, "Live({})", local_id),
            MirStatementKind::StorageDead(local_id) => write!(f, "Dead({})", local_id),

            _ => write!(f, "{:?}", self.kind),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PlaceKind {
    Local(LocalDeclId),
    Global(GlobalDeclId),
}

#[derive(Debug, Clone)]
pub struct Place {
    pub kind: PlaceKind,
    pub projection: Vec<Projection>,
}

impl Place {
    pub fn new(kind: PlaceKind) -> Self {
        Self {
            kind,
            projection: Vec::new(),
        }
    }

    pub fn local(local_id: LocalDeclId) -> Self {
        Self::new(PlaceKind::Local(local_id))
    }

    pub fn global(global_id: GlobalDeclId) -> Self {
        Self::new(PlaceKind::Global(global_id))
    }
}

impl Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            match &self.kind {
                PlaceKind::Local(local_id) => local_id.to_string(),
                PlaceKind::Global(global_id) => global_id.to_string(),
            },
            ternary!(
                self.projection.is_empty(),
                String::new(),
                format!(".{:?}", self.projection)
            )
        )
    }
}

#[derive(Debug, Clone)]
pub enum Projection {
    Deref,
    Field(usize, TyId),
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
    Ref(RefKind, Place),

    FnRef(DefId),
    AnonFn {
        body_id: MirBodyId,
        captures: Vec<Operand>,
    },

    // Len(Location),
    // Cast(CastKind, Operand, Ty),
    Binary(BinaryOp, Box<(Operand, Operand)>),
    // UnaryOp(UnOp, Operand),
    // NullaryOp(NullOp, Ty),
    Discriminant(Place),
    Aggregate(TyId, Vec<Operand>),
}

impl Display for RValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Use(op) => write!(f, "{}", &op),

            Self::Aggregate(def_id, operands) => write!(
                f,
                "Aggregate({:?}, [{}])",
                def_id,
                operands
                    .iter()
                    .map(|op| op.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),

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
    Copy(Place),
    Move(Place),
    Const(ConstId),
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Copy(loc) => write!(f, "copy {}", &loc),
            Self::Move(loc) => write!(f, "move {}", &loc),
            Self::Const(id) => write!(f, "{:?}", &id),
        }
    }
}
