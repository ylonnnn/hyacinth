use hycc_span::Span;
use hycc_ty::context::TyId;

#[derive(Debug, Clone)]
pub enum LocalDeclKind {
    Ret,
    Param,
    Var,
    Temp,
}

pub type Mutability = hycc_hir::HirMutability;

#[derive(Debug, Clone)]
pub struct LocalDecl {
    pub ty: TyId,
    pub span: Span,
    pub kind: LocalDeclKind,
    pub mutability: Mutability,
}

impl LocalDecl {
    pub fn new(kind: LocalDeclKind, ty: TyId, mutability: Mutability, span: Span) -> Self {
        Self {
            ty,
            span,
            kind,
            mutability,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalDeclId(pub(crate) usize);

impl LocalDeclId {
    #[allow(non_upper_case_globals)]
    pub const Invalid: Self = Self(usize::MAX);

    pub fn unwrap(&self) -> usize {
        assert_ne!(self.0, usize::MAX, "local decl id is not valid!");
        self.0
    }
}
