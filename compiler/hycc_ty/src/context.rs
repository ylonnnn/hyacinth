use std::collections::HashMap;

use hycc_hir::{
    HirId,
    def::{BuiltinIntTy, BuiltinTyKind, DefId},
};

use crate::ty::{IntTy, TyKind};

#[derive(Debug)]
pub struct TyCtx {
    storage: Vec<TyKind>,
    map: HashMap<TyKind, TyId>,

    node_ty_map: HashMap<HirId, TyId>,
    def_ty_map: HashMap<DefId, TyId>,
}

impl TyCtx {
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
            map: HashMap::new(),

            node_ty_map: HashMap::new(),
            def_ty_map: HashMap::new(),
        }
    }

    pub fn intern(&mut self, ty: TyKind) -> TyId {
        if let Some(ty_id) = self.map.get(&ty) {
            return *ty_id;
        }

        let ty_id = TyId(self.storage.len());

        self.map.insert(ty.clone(), ty_id);
        self.storage.push(ty);

        ty_id
    }

    pub fn get(&self, ty_id: TyId) -> &TyKind {
        &self.storage[ty_id.unwrap()]
    }

    pub fn attach_to_hir(&mut self, hir_id: HirId, ty_id: TyId) {
        self.node_ty_map.insert(hir_id, ty_id);
    }

    pub fn get_ty_of_hir(&self, hir_id: HirId) -> Option<TyId> {
        self.node_ty_map.get(&hir_id).map(|t| *t)
    }

    pub fn attach_to_def(&mut self, def_id: DefId, ty_id: TyId) {
        self.def_ty_map.insert(def_id, ty_id);
    }

    pub fn get_ty_of_def(&self, def_id: DefId) -> Option<TyId> {
        self.def_ty_map.get(&def_id).map(|t| *t)
    }

    pub fn make_builtin_ty(&mut self, kind: &BuiltinTyKind) -> TyId {
        match kind {
            BuiltinTyKind::Int(kind) => match kind {
                BuiltinIntTy::Fixed(width, signed) => {
                    self.make_int_ty(IntTy::Fixed(*width, *signed))
                }
                BuiltinIntTy::Size(signed) => self.make_int_ty(IntTy::Size(*signed)),
            },

            BuiltinTyKind::Float(width) => self.make_float_ty(*width),
            BuiltinTyKind::Bool => self.make_bool_ty(),
            BuiltinTyKind::Char => self.make_char_ty(),
            BuiltinTyKind::String => self.make_string_ty(),
        }
    }

    pub fn make_int_ty(&mut self, ty: IntTy) -> TyId {
        self.intern(TyKind::Int(ty))
    }

    pub fn make_float_ty(&mut self, width: u8) -> TyId {
        self.intern(TyKind::Float(width))
    }

    pub fn make_bool_ty(&mut self) -> TyId {
        self.intern(TyKind::Bool)
    }

    pub fn make_char_ty(&mut self) -> TyId {
        self.intern(TyKind::Char)
    }

    pub fn make_string_ty(&mut self) -> TyId {
        self.intern(TyKind::String)
    }

    pub fn make_adt_ty(&mut self, def_id: DefId) -> TyId {
        self.intern(TyKind::Adt(def_id))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TyId(usize);

impl TyId {
    #[allow(non_upper_case_globals)]
    pub const Invalid: Self = Self(usize::MAX);

    pub fn unwrap(&self) -> usize {
        assert_ne!(self.0, usize::MAX, "type id is not valid!");
        self.0
    }
}
