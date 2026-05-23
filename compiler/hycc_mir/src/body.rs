use std::fmt::Display;

use hycc_span::Span;
use hycc_ty::context::TyId;

use crate::{
    basic_block::{MirBasicBlock, MirBasicBlockId},
    local::{LocalDecl, LocalDeclId, LocalDeclKind, Mutability},
};

#[derive(Debug, Clone)]
pub struct MirBody {
    pub basic_blocks: Vec<MirBasicBlock>,
    pub(crate) local_decls: Vec<LocalDecl>,
}

impl MirBody {
    pub fn new() -> Self {
        Self {
            basic_blocks: Vec::new(),
            local_decls: Vec::new(),
        }
    }

    pub fn insert(&mut self, basic_block: MirBasicBlock) -> MirBasicBlockId {
        self.basic_blocks.push(basic_block);
        MirBasicBlockId(self.basic_blocks.len() - 1)
    }

    pub fn get(&self, id: MirBasicBlockId) -> &MirBasicBlock {
        &self.basic_blocks[id.unwrap()]
    }

    pub fn get_mut(&mut self, id: MirBasicBlockId) -> &mut MirBasicBlock {
        &mut self.basic_blocks[id.unwrap()]
    }

    pub fn declare_local(&mut self, decl: LocalDecl) -> LocalDeclId {
        self.local_decls.push(decl);
        LocalDeclId(self.local_decls.len() - 1)
    }

    pub fn get_local(&self, id: LocalDeclId) -> &LocalDecl {
        &self.local_decls[id.unwrap()]
    }

    pub fn declare_local_ret(&mut self, ty: TyId) -> LocalDeclId {
        self.declare_local(LocalDecl::new(
            LocalDeclKind::Ret,
            ty,
            Mutability::Mutable,
            Span::default(),
        ))
    }

    pub fn declare_local_param(
        &mut self,
        ty: TyId,
        mutability: Mutability,
        span: Span,
    ) -> LocalDeclId {
        self.declare_local(LocalDecl::new(LocalDeclKind::Param, ty, mutability, span))
    }

    pub fn declare_local_var(
        &mut self,
        ty: TyId,
        mutability: Mutability,
        span: Span,
    ) -> LocalDeclId {
        self.declare_local(LocalDecl::new(LocalDeclKind::Var, ty, mutability, span))
    }

    pub fn declare_local_temp(&mut self, ty: TyId, span: Span) -> LocalDeclId {
        self.declare_local(LocalDecl::new(
            LocalDeclKind::Temp,
            ty,
            Mutability::Immutable,
            span,
        ))
    }
}

impl Display for MirBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, basic_block) in self.basic_blocks.iter().enumerate() {
            writeln!(f, "bb{}:\n{}", i, &basic_block)?;
        }

        Ok(())
    }
}
