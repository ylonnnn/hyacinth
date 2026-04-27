use std::{collections::HashMap, sync::Arc};

use hycc_hir::{
    HirId,
    def::{BuiltinIntTy, BuiltinTyKind, DefId},
};
use hycc_util::bug;

use crate::ty::{FnTy, InferKind, IntTy, RefMutability, Ty, TyKind, TyVar};

#[derive(Debug)]
pub struct TyCtx {
    storage: Vec<TyKind>,
    map: HashMap<TyKind, TyId>,

    vars: Vec<TyVar>,

    node_ty_map: HashMap<HirId, Ty>,
    def_ty_map: HashMap<DefId, Ty>,
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

    pub fn get_mut(&mut self, ty_id: TyId) -> &mut TyKind {
        &mut self.storage[ty_id.unwrap()]
    }

    pub fn hir_ids(&self) -> Vec<HirId> {
        self.node_ty_map.keys().cloned().collect()
    }

    pub fn hir_tys(&self) -> Vec<(HirId, &Ty)> {
        self.node_ty_map
            .iter()
            .map(|(hir_id, ty)| (*hir_id, ty))
            .collect()
    }

    pub fn def_ids(&self) -> Vec<DefId> {
        self.def_ty_map.keys().cloned().collect()
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
            TyKind::Array(inner_ty) => {
                let inner_ty = *inner_ty;
                let inner_ty_id = self.resolve_ty(inner_ty);

                if inner_ty == inner_ty_id {
                    ty_id
                } else {
                    self.intern(TyKind::Array(inner_ty_id))
                }
            }

            TyKind::Slice(inner_ty) => {
                let inner_ty = *inner_ty;
                let inner_ty_id = self.resolve_ty(inner_ty);

                if inner_ty == inner_ty_id {
                    ty_id
                } else {
                    self.intern(TyKind::Slice(inner_ty_id))
                }
            }

            TyKind::Ref(inner_ty, mutability) => {
                let (inner_ty, mutability) = (*inner_ty, *mutability);
                let inner_ty_id = self.resolve_ty(inner_ty);

                if inner_ty == inner_ty_id {
                    ty_id
                } else {
                    self.intern(TyKind::Ref(inner_ty_id, mutability))
                }
            }

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
            (TyVar::Bound(a_ty), TyVar::Bound(b_ty)) => {
                self.unify_ty(*a_ty, *b_ty);
            }
            (_, TyVar::Bound(v)) => self.bind_var(a, *v),
            (TyVar::Bound(v), _) => self.bind_var(b, *v),
            (TyVar::Unbound, TyVar::Unbound) => self.vars[a.unwrap()] = TyVar::Linked(b),
            (_, TyVar::Linked(..)) | (TyVar::Linked(..), _) => {
                panic!("resolve_var should eliminate links");
            }
        }
    }

    pub fn unify_ty(&mut self, a: TyId, b: TyId) -> bool {
        if a == b {
            return true;
        }

        let a_ty = &self.storage[a.unwrap()];
        let b_ty = &self.storage[b.unwrap()];

        match (&a_ty, &b_ty) {
            (other, TyKind::Infer(v_id, kind)) if kind.compatible(&other) => {
                self.bind_var(*v_id, a);
                true
            }
            (TyKind::Infer(v_id, kind), other) if kind.compatible(&other) => {
                self.bind_var(*v_id, b);
                true
            }

            (TyKind::Array(a_inner), TyKind::Array(b_inner)) => self.unify_ty(*a_inner, *b_inner),
            (TyKind::Slice(a_inner), TyKind::Slice(b_inner)) => self.unify_ty(*a_inner, *b_inner),

            (TyKind::Tuple(a_tys), TyKind::Tuple(b_tys)) => a_tys
                .clone()
                .iter()
                .zip(b_tys.clone().iter())
                .all(|(a_ty, b_ty)| self.unify_ty(*a_ty, *b_ty)),

            (TyKind::Ref(a_inner, a_mut), TyKind::Ref(b_inner, b_mut)) => {
                let mut_valid = *a_mut == RefMutability::Immutable || *a_mut == *b_mut;
                mut_valid && self.unify_ty(*a_inner, *b_inner)
            }

            // (TyKind::Adt(a_inner), TyKind::Adt(b_inner)) => self.unify_ty(*a_inner, *b_inner),
            (_, _) => false,
        }
    }

    pub fn attach_to_hir(&mut self, hir_id: HirId, ty: Ty) {
        self.node_ty_map.insert(hir_id, ty);
    }

    pub fn get_ty_of_hir(&self, hir_id: HirId) -> Option<&Ty> {
        self.node_ty_map.get(&hir_id)
    }

    pub fn get_mut_ty_of_hir(&mut self, hir_id: HirId) -> Option<&mut Ty> {
        self.node_ty_map.get_mut(&hir_id)
    }

    pub fn attach_to_def(&mut self, def_id: DefId, ty: Ty) {
        self.def_ty_map.insert(def_id, ty);
    }

    pub fn get_ty_of_def(&self, def_id: DefId) -> Option<&Ty> {
        self.def_ty_map.get(&def_id)
    }

    pub fn get_mut_ty_of_def(&mut self, def_id: DefId) -> Option<&mut Ty> {
        self.def_ty_map.get_mut(&def_id)
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

            BuiltinTyKind::Infer => self.make_inferred_ty(InferKind::Any),
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

    pub fn make_array_ty(&mut self, inner_ty: TyId) -> TyId {
        self.intern(TyKind::Array(inner_ty))
    }

    pub fn make_slice_ty(&mut self, inner_ty: TyId) -> TyId {
        self.intern(TyKind::Slice(inner_ty))
    }

    pub fn make_tuple_ty(&mut self, tys: Arc<[TyId]>) -> TyId {
        self.intern(TyKind::Tuple(Box::new(tys)))
    }

    pub fn make_ref_ty(&mut self, inner_ty: TyId, mutability: RefMutability) -> TyId {
        self.intern(TyKind::Ref(inner_ty, mutability))
    }

    pub fn make_fn_ty(&mut self, params: Arc<[TyId]>, ret_ty: TyId) -> TyId {
        self.intern(TyKind::Fn(Box::new(FnTy { params, ret_ty })))
    }

    pub fn make_adt_ty(&mut self, def_id: DefId) -> TyId {
        self.intern(TyKind::Adt(def_id))
    }

    pub fn make_inferred_ty(&mut self, kind: InferKind) -> TyId {
        let var_id = self.fresh_var();
        self.storage.push(TyKind::Infer(var_id, kind));
        TyId(self.storage.len() - 1)
    }

    pub fn is_inferred(&self, ty_id: TyId) -> bool {
        let kind = self.get(ty_id);

        match kind {
            TyKind::Infer(_, InferKind::Any) => true,

            TyKind::Array(inner) => self.is_inferred(*inner),
            TyKind::Slice(inner) => self.is_inferred(*inner),

            TyKind::Ref(inner, _) => self.is_inferred(*inner),

            _ => false,
        }
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
