use std::fmt::Display;

use hycc_span::Span;
use hycc_ty::context::TyId;

use crate::{
    basic_block::{MirBasicBlock, MirBasicBlockId},
    local::{LocalDecl, LocalDeclId, LocalDeclKind, Mutability},
    stmt::MirStatement,
    term::MirTerminator,
};

#[derive(Debug, Clone)]
pub struct MirBody {
    pub basic_blocks: Vec<MirBasicBlock>,
    pub(crate) local_decls: Vec<LocalDecl>,

    new_bb: bool,
}

impl MirBody {
    pub fn new() -> Self {
        Self {
            basic_blocks: Vec::new(),
            local_decls: Vec::new(),

            new_bb: true,
        }
    }

    pub fn insert(&mut self, basic_block: MirBasicBlock) -> MirBasicBlockId {
        self.new_bb = false;

        self.basic_blocks.push(basic_block);
        MirBasicBlockId(self.basic_blocks.len() - 1)
    }

    pub fn get(&self, id: MirBasicBlockId) -> &MirBasicBlock {
        &self.basic_blocks[id.unwrap()]
    }

    pub fn get_mut(&mut self, id: MirBasicBlockId) -> &mut MirBasicBlock {
        &mut self.basic_blocks[id.unwrap()]
    }

    pub fn cue(&mut self) {
        self.new_bb = true;
    }

    pub fn flush(&mut self) {
        self.new_bb = false;
    }

    pub fn current_bb(&self) -> MirBasicBlockId {
        MirBasicBlockId(self.basic_blocks.len() - 1)
    }

    pub fn insert_stmt(&mut self, stmt: MirStatement) {
        if self.new_bb {
            self.insert(MirBasicBlock::new());
            self.new_bb = false;
        }

        self.basic_blocks.last_mut().unwrap().statements.push(stmt);
    }

    pub fn attach_term(&mut self, term: MirTerminator) {
        if self.new_bb {
            self.insert(MirBasicBlock::new());
            self.new_bb = false;
        }

        self.basic_blocks
            .last_mut()
            .unwrap()
            .terminator
            .replace(term);
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
