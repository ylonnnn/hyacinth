use std::collections::HashMap;

use hycc_hir::def::DefId;
use hycc_span::Span;
use hycc_ty::context::TyId;

use crate::{
    decl::{Decl, GlobalDeclId, LocalDeclId, Mutability},
    table::MirTable,
};

#[derive(Debug, Clone, Copy)]
pub enum MirDef {
    Local(LocalDeclId),
    Global(GlobalDeclId),
    Body(DefId),
}

#[derive(Debug)]
pub struct MirLoweringCtx {
    pub table: MirTable,
    global_decls: Vec<Decl>,
    def_map: HashMap<DefId, MirDef>,
}

impl MirLoweringCtx {
    pub fn new() -> Self {
        Self {
            table: MirTable::new(),
            global_decls: Vec::new(),
            def_map: HashMap::new(),
        }
    }

    pub fn declare_global(&mut self, ty: TyId, mutability: Mutability, span: Span) -> GlobalDeclId {
        self.global_decls.push(Decl::global(ty, mutability, span));
        GlobalDeclId(self.global_decls.len() - 1)
    }

    pub fn define(&mut self, def_id: DefId, def: MirDef) {
        self.def_map.insert(def_id, def);
    }

    pub fn get_def(&mut self, def_id: DefId) -> MirDef {
        *self.def_map.get(&def_id).unwrap()
    }
}
