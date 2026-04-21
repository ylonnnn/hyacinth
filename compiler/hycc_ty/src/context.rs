use std::collections::HashMap;

use hycc_hir::{
    HirId,
    def::{BuiltinIntTy, BuiltinTyKind, DefId},
};
use hycc_util::bug;

use crate::ty::{InferKind, IntTy, TyKind, TyVar};

#[derive(Debug)]
pub struct TyCtx {
    storage: Vec<TyKind>,
    map: HashMap<TyKind, TyId>,

    vars: Vec<TyVar>,

    node_ty_map: HashMap<HirId, TyId>,
    def_ty_map: HashMap<DefId, TyId>,
}

impl TyCtx {
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
            map: HashMap::new(),

            vars: Vec::new(),

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

    pub fn fresh_var(&mut self) -> TyVarId {
        self.vars.push(TyVar::Unbound);
        TyVarId(self.vars.len() - 1)
    }

    pub fn resolve_var(&mut self, var_id: TyVarId) -> TyVarId {
        if let Some(TyVar::Linked(id)) = self.vars.get(var_id.unwrap()) {
            let root = self.resolve_var(*id);
            self.vars[var_id.unwrap()] = TyVar::Linked(root);
            root
        } else {
            var_id
        }
    }

    pub fn bind_var(&mut self, var_id: TyVarId, ty_id: TyId) {
        let root = self.resolve_var(var_id);

        // TODO: check infinite types

        self.vars[root.unwrap()] = TyVar::Bound(ty_id);
    }

    pub fn resolve_ty(&mut self, ty_id: TyId) -> TyId {
        let Some(ty) = self.storage.get(ty_id.unwrap()) else {
            bug!("no type stored for ty_id: {ty_id:?}");
        };

        match &ty {
            TyKind::Infer(var_id, _) => {
                let root = self.resolve_var(*var_id);
                match &self.vars[root.unwrap()] {
                    TyVar::Bound(ty_id) => self.resolve_ty(*ty_id),
                    _ => ty_id,
                }
            }

            _ => ty_id,
        }
    }

    pub fn unify(&mut self, a: TyVarId, b: TyVarId) {
        let a = self.resolve_var(a);
        let b = self.resolve_var(b);

        match (&self.vars[a.unwrap()], &self.vars[b.unwrap()]) {
            (TyVar::Bound(a_ty), TyVar::Bound(b_ty)) => self.unify_ty(*a_ty, *b_ty),
            (_, TyVar::Bound(v)) => self.bind_var(a, *v),
            (TyVar::Bound(v), _) => self.bind_var(b, *v),
            (TyVar::Unbound, TyVar::Unbound) => self.vars[a.unwrap()] = TyVar::Linked(b),
            (_, TyVar::Linked(..)) | (TyVar::Linked(..), _) => {
                panic!("resolve_var should eliminate links");
            }
        }
    }

    pub fn unify_ty(&mut self, a: TyId, b: TyId) {
        if a == b {
            return;
        }

        let a_ty = &self.storage[a.unwrap()];
        let b_ty = &self.storage[b.unwrap()];

        match (&a_ty, &b_ty) {
            (other, TyKind::Infer(v_id, kind)) if kind.compatible(&other) => {
                self.bind_var(*v_id, a)
            }
            (TyKind::Infer(v_id, kind), other) if kind.compatible(&other) => {
                self.bind_var(*v_id, b)
            }
            // (TyKind::Adt(a_inner), TyKind::Adt(b_inner)) => self.unify_ty(*a_inner, *b_inner),
            (a, b) => {
                // println!("{a:?} {b:?}")
                panic!("type mismatch: {a:?} {b:?}");
            }
        }
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
            BuiltinTyKind::Unit => self.make_unit_ty(),

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

    pub fn make_unit_ty(&mut self) -> TyId {
        self.intern(TyKind::Unit)
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

    pub fn make_inferred_ty(&mut self, kind: InferKind) -> TyId {
        let var_id = self.fresh_var();
        self.intern(TyKind::Infer(var_id, kind))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TyVarId(usize);

impl TyVarId {
    #[allow(non_upper_case_globals)]
    pub const Invalid: Self = Self(usize::MAX);

    pub fn unwrap(&self) -> usize {
        assert_ne!(self.0, usize::MAX, "type variable id is not valid!");
        self.0
    }
}
