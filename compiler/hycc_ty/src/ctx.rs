use std::{
    collections::{HashMap, HashSet},
    iter::FilterMap,
    sync::Arc,
};

use hycc_hir::{
    HirId,
    def::{BuiltinIntTy, BuiltinTyKind, DefId},
    path::HirIdentArguments,
};
use hycc_span::Span;
use hycc_util::{bug, ternary};

use crate::{
    extension::{ExtNominalTargetKind, ExtTargetKind, ExtensionTable},
    ty::{
        FnTy, GenericArg, InferKind, IntTy, ParamTy, RefMutability, Ty, TyKind, TyVar, TyVarKind,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TyResState {
    Inferred(TyId),
    Resolved(TyId),
    Unresolved,
    Resolving,
}

#[derive(Debug)]
pub struct TyCtx {
    pub ext_table: ExtensionTable,

    res_state: HashMap<HirId, TyResState>,
    hir_ty_map: HashMap<HirId, Ty>,
    def_ty_map: HashMap<DefId, Ty>,
    ty_def_map: HashMap<TyId, DefId>,

    map: HashMap<TyKind, TyId>,
    storage: Vec<TyKind>,

    vars: Vec<TyVar>,
}

impl TyCtx {
    pub fn new() -> Self {
        Self {
            ext_table: ExtensionTable::new(),

            res_state: HashMap::new(),
            hir_ty_map: HashMap::new(),
            def_ty_map: HashMap::new(),
            ty_def_map: HashMap::new(),

            storage: Vec::new(),
            map: HashMap::new(),

            vars: Vec::new(),
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
        self.hir_ty_map.keys().cloned().collect()
    }

    pub fn hir_tys(&self) -> Vec<(HirId, &Ty)> {
        self.hir_ty_map
            .iter()
            .map(|(hir_id, ty)| (*hir_id, ty))
            .collect()
    }

    pub fn def_ids(&self) -> Vec<DefId> {
        self.def_ty_map.keys().cloned().collect()
    }

    pub fn get_var(&self, var_id: TyVarId) -> &TyVar {
        &self.vars[var_id.unwrap()]
    }

    pub fn fresh_var(&mut self, span: Span) -> TyVarId {
        self.vars.push(TyVar::new(span, TyVarKind::Unbound));
        TyVarId(self.vars.len() - 1)
    }

    pub fn resolve_var(&mut self, var_id: TyVarId) -> TyVarId {
        let Some(var) = self.vars.get(var_id.unwrap()) else {
            return var_id;
        };

        if let TyVarKind::Linked(id) = &var.kind {
            let root = self.resolve_var(*id);
            self.vars[var_id.unwrap()].kind = TyVarKind::Linked(root);
            root
        } else {
            var_id
        }
    }

    pub fn bind_var(&mut self, var_id: TyVarId, ty_id: TyId) {
        let root = self.resolve_var(var_id);

        // TODO: check infinite types

        self.vars[root.unwrap()].kind = TyVarKind::Bound(ty_id);
    }

    pub fn resolve_ty(&mut self, ty_id: TyId) -> TyId {
        let Some(ty) = self.storage.get(ty_id.unwrap()) else {
            bug!("no type stored for ty_id: {ty_id:?}");
        };

        match &ty {
            TyKind::Array(inner_ty) => {
                let inner_ty = *inner_ty;
                let inner_ty_id = self.resolve_ty(inner_ty);

                ternary!(
                    inner_ty == inner_ty_id,
                    ty_id,
                    self.intern(TyKind::Array(inner_ty_id))
                )
            }

            TyKind::Slice(inner_ty) => {
                let inner_ty = *inner_ty;
                let inner_ty_id = self.resolve_ty(inner_ty);

                ternary!(
                    inner_ty == inner_ty_id,
                    ty_id,
                    self.intern(TyKind::Slice(inner_ty_id))
                )
            }

            TyKind::Tuple(tys) => {
                let mut updated = false;
                let tys = tys
                    .clone()
                    .iter()
                    .map(|ty| {
                        let res = self.resolve_ty(*ty);
                        (res, updated = updated || res != *ty).0
                    })
                    .collect::<Vec<_>>();

                ternary!(updated, self.make_tuple_ty(tys.into()), ty_id)
            }

            TyKind::Ref(inner_ty, mutability) => {
                let (inner_ty, mutability) = (*inner_ty, *mutability);
                let inner_ty_id = self.resolve_ty(inner_ty);

                ternary!(
                    inner_ty == inner_ty_id,
                    ty_id,
                    self.intern(TyKind::Ref(inner_ty_id, mutability))
                )
            }

            TyKind::Fn(func, arguments) => {
                let f_def_id = func.def_id;
                let f_params = func.params.clone();
                let f_ret_ty = func.ret_ty;

                let mut updated = false;

                let generic_args = arguments.clone();
                let arguments = generic_args
                    .iter()
                    .map(|arg| match &arg {
                        GenericArg::Ty(ty_id) => {
                            let r_ty_id = self.resolve_ty(*ty_id);
                            (
                                GenericArg::Ty(r_ty_id),
                                updated = updated || r_ty_id != *ty_id,
                            )
                                .0
                        }
                    })
                    .collect::<Vec<_>>();

                let params = f_params
                    .iter()
                    .map(|param| {
                        let ty_id = self.resolve_ty(*param);
                        (ty_id, updated = updated || ty_id != *param).0
                    })
                    .collect::<Vec<_>>();

                let ret_ty = {
                    let ty_id = self.resolve_ty(f_ret_ty);
                    (ty_id, updated = updated || ty_id != f_ret_ty).0
                };

                ternary!(
                    updated,
                    self.make_fn_ty(arguments.into(), f_def_id, params.into(), ret_ty),
                    ty_id
                )
            }

            TyKind::Adt(def_id, args) => {
                let (def_id, args) = (*def_id, args.clone());
                let mut updated = false;

                let args = args
                    .iter()
                    .map(|arg| match &arg {
                        GenericArg::Ty(ty_id) => {
                            let r_ty_id = self.resolve_ty(*ty_id);
                            GenericArg::Ty((r_ty_id, updated = updated || r_ty_id != *ty_id).0)
                        }
                    })
                    .collect::<Vec<_>>();

                ternary!(updated, self.make_adt_ty(def_id, args.into()), ty_id)
            }

            TyKind::Infer(var_id, _) => {
                let root = self.resolve_var(*var_id);

                match &self.vars[root.unwrap()].kind {
                    TyVarKind::Bound(ty_id) => self.resolve_ty(*ty_id),
                    _ => ty_id,
                }
            }

            _ => ty_id,
        }
    }

    pub fn instantiate(&mut self, ty_id: TyId, args: &[&[GenericArg]]) -> TyId {
        let ty_id = self.resolve_ty(ty_id);
        match self.get(ty_id) {
            TyKind::Array(ty_id) => {
                let ty_id = self.instantiate(*ty_id, args.into());
                self.make_array_ty(ty_id)
            }

            TyKind::Slice(ty_id) => {
                let ty_id = self.instantiate(*ty_id, args.into());
                self.make_slice_ty(ty_id)
            }

            TyKind::Tuple(tys) => {
                let tys = tys
                    .clone()
                    .iter()
                    .map(|ty| self.instantiate(*ty, args.into()))
                    .collect::<Arc<_>>();
                self.make_tuple_ty(tys)
            }

            TyKind::Ref(ty_id, mutability) => {
                let mutability = *mutability;
                let ty_id = self.instantiate(*ty_id, args.into());

                self.make_ref_ty(ty_id, mutability)
            }

            TyKind::Fn(fn_ty, g_args) => {
                let (def_id, ret_ty) = (fn_ty.def_id, fn_ty.ret_ty);
                let params = fn_ty.params.clone();
                let g_args = g_args.clone();

                let params = params
                    .clone()
                    .iter()
                    .map(|param| self.instantiate(*param, args.into()))
                    .collect::<Arc<_>>();
                let ret_ty = self.instantiate(ret_ty, args.into());

                let new_args = g_args
                    .iter()
                    .map(|arg| match arg {
                        GenericArg::Ty(ty_id) => {
                            GenericArg::Ty(self.instantiate(*ty_id, args.into()))
                        }
                    })
                    .collect::<Arc<_>>();

                self.make_fn_ty(new_args, def_id, params, ret_ty)
            }

            TyKind::Adt(def_id, g_args) => {
                let def_id = *def_id;
                let new_args = g_args
                    .clone()
                    .iter()
                    .map(|arg| match arg {
                        GenericArg::Ty(ty_id) => {
                            GenericArg::Ty(self.instantiate(*ty_id, args.into()))
                        }
                    })
                    .collect::<Arc<_>>();
                self.make_adt_ty(def_id, new_args)
            }

            TyKind::Param(param) => {
                let n = args.len();
                match args
                    .get(param.depth().saturating_sub(1) as usize)
                    .and_then(|args| args.get(param.idx() as usize))
                {
                    Some(GenericArg::Ty(ty_id)) => *ty_id,
                    _ => ty_id,
                }
            }

            _ => ty_id,
        }
    }

    pub fn ext_target_kind_of(&self, ty_id: TyId) -> ExtTargetKind {
        self.get_ty_def_id(ty_id).map_or_else(
            || match &self.get(ty_id) {
                TyKind::Fn(..) => todo!("allow fn type extensions"),

                TyKind::Array(..) => ExtTargetKind::Array,
                TyKind::Slice(..) => ExtTargetKind::Slice,
                TyKind::Tuple(tup) => ExtTargetKind::Tuple(tup.len()),
                TyKind::Ref(..) => ExtTargetKind::Ref,

                // TODO: improve?: allow ambiguous inferred types (except `Any`) to be used for
                // other concrete type extensions
                TyKind::Infer(_, kind) => ExtTargetKind::Nominal(ExtNominalTargetKind::Blanket),

                _ => bug!("other type kinds are expected to be defined/have a definition."),
            },
            |def_id| ExtTargetKind::Nominal(ExtNominalTargetKind::Def(def_id)),
        )
    }

    pub fn unify(&mut self, a: TyVarId, b: TyVarId) {
        let a = self.resolve_var(a);
        let b = self.resolve_var(b);

        match (&self.vars[a.unwrap()].kind, &self.vars[b.unwrap()].kind) {
            (TyVarKind::Bound(a_ty), TyVarKind::Bound(b_ty)) => {
                self.unify_ty(*a_ty, *b_ty);
            }
            (_, TyVarKind::Bound(v)) => self.bind_var(a, *v),
            (TyVarKind::Bound(v), _) => self.bind_var(b, *v),
            (TyVarKind::Unbound, TyVarKind::Unbound) => {
                self.vars[a.unwrap()].kind = TyVarKind::Linked(b)
            }
            (_, TyVarKind::Linked(..)) | (TyVarKind::Linked(..), _) => {
                bug!("resolve_var must eliminate links");
            }
        }
    }

    pub fn unify_ty(&mut self, a: TyId, b: TyId) -> bool {
        let (res_a, res_b) = (self.resolve_ty(a), self.resolve_ty(b));
        if res_a == res_b {
            return true;
        }

        let a_ty = &self.storage[res_a.unwrap()];
        let b_ty = &self.storage[res_b.unwrap()];

        match (&a_ty, &b_ty) {
            (other, TyKind::Infer(v_id, kind)) if kind.compatible(&other) => {
                self.bind_var(*v_id, res_a);
                true
            }
            (TyKind::Infer(v_id, kind), other) if kind.compatible(&other) => {
                self.bind_var(*v_id, res_b);
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

            (TyKind::Fn(a_func, a_args), TyKind::Fn(b_func, b_args)) => {
                if a_func.params.len() != b_func.params.len() {
                    return false;
                }

                let (a_params, b_params) = (a_func.params.clone(), b_func.params.clone());
                let (a_ret, b_ret) = (a_func.ret_ty, b_func.ret_ty);

                if !a_args
                    .clone()
                    .into_iter()
                    .zip(b_args.clone().into_iter())
                    .all(|(a_arg, b_arg)| match (a_arg, b_arg) {
                        (GenericArg::Ty(a_ty_id), GenericArg::Ty(b_ty_id)) => {
                            self.unify_ty(*a_ty_id, *b_ty_id)
                        }
                    })
                {
                    return false;
                }

                a_params
                    .iter()
                    .zip(b_params.iter())
                    .all(|(ap_ty, bp_ty)| self.unify_ty(*ap_ty, *bp_ty))
                    && self.unify_ty(a_ret, b_ret)
            }

            (TyKind::Never, _) | (_, TyKind::Never) => true,

            (TyKind::Adt(a_def, a_args), TyKind::Adt(b_def, b_args)) => {
                if a_def != b_def || a_args.len() != b_args.len() {
                    return false;
                }

                let (a_args, b_args) = (a_args.clone(), b_args.clone());
                a_args
                    .iter()
                    .zip(b_args.iter())
                    .all(|(a, b)| match (&a, &b) {
                        (GenericArg::Ty(a_ty), GenericArg::Ty(b_ty)) => self.unify_ty(*a_ty, *b_ty),
                    })
            }
            (_, _) => false,
        }
    }

    pub fn attach_to_hir(&mut self, hir_id: HirId, ty: Ty) {
        self.res_state.insert(hir_id, TyResState::Resolved(ty.id));
        self.hir_ty_map.insert(hir_id, ty);
    }

    pub fn dettach_hir(&mut self, hir_id: HirId) {
        self.hir_ty_map.remove(&hir_id);
    }

    pub fn update_hir_res_state(&mut self, hir_id: HirId, state: TyResState) {
        self.res_state.insert(hir_id, state);
    }

    pub fn get_hir_res_state(&self, hir_id: HirId) -> Option<TyResState> {
        self.res_state.get(&hir_id).cloned()
    }

    pub fn expect_hir_res_state(&self, hir_id: HirId) -> TyResState {
        self.get_hir_res_state(hir_id).unwrap_or_else(|| {
            panic!("expected a resolution state attached to type of hir {hir_id:?}")
        })
    }

    pub fn get_hir_ty(&self, hir_id: HirId) -> Option<&Ty> {
        self.hir_ty_map.get(&hir_id)
    }

    pub fn get_hir_mut_ty(&mut self, hir_id: HirId) -> Option<&mut Ty> {
        self.hir_ty_map.get_mut(&hir_id)
    }

    pub fn get_hir_ty_id(&self, hir_id: HirId) -> Option<TyId> {
        self.get_hir_ty(hir_id).map(|ty| ty.id)
    }

    pub fn expect_hir_ty(&self, hir_id: HirId) -> &Ty {
        self.get_hir_ty(hir_id)
            .unwrap_or_else(|| panic!("expected a type attached to hir id {hir_id:?}"))
    }

    pub fn expect_hir_mut_ty(&mut self, hir_id: HirId) -> &mut Ty {
        self.get_hir_mut_ty(hir_id)
            .unwrap_or_else(|| panic!("expected a type attached to hir id {hir_id:?}"))
    }

    pub fn expect_hir_ty_id(&self, hir_id: HirId) -> TyId {
        self.expect_hir_ty(hir_id).id
    }

    pub fn attach_to_def(&mut self, def_id: DefId, ty: Ty) {
        self.ty_def_map.insert(ty.id, def_id);
        self.def_ty_map.insert(def_id, ty);
    }

    pub fn dettach_def(&mut self, def_id: DefId) {
        self.def_ty_map.remove(&def_id);
    }

    pub fn get_def_ty_id(&self, def_id: DefId) -> Option<TyId> {
        self.def_ty_map.get(&def_id).map(|ty| ty.id)
    }

    pub fn get_def_ty(&self, def_id: DefId) -> Option<&Ty> {
        self.def_ty_map.get(&def_id)
    }

    pub fn get_def_mut_ty(&mut self, def_id: DefId) -> Option<&mut Ty> {
        self.def_ty_map.get_mut(&def_id)
    }

    pub fn expect_def_ty_id(&self, def_id: DefId) -> TyId {
        self.get_def_ty_id(def_id)
            .unwrap_or_else(|| panic!("expected a ty attached to {def_id:?}"))
    }

    pub fn expect_def_ty(&self, def_id: DefId) -> &Ty {
        self.get_def_ty(def_id)
            .unwrap_or_else(|| panic!("expected a ty attached to {def_id:?}"))
    }

    pub fn expect_def_mut_ty(&mut self, def_id: DefId) -> &mut Ty {
        self.get_def_mut_ty(def_id)
            .unwrap_or_else(|| panic!("expected a ty attached to {def_id:?}"))
    }

    pub fn get_ty_def_id(&self, ty_id: TyId) -> Option<DefId> {
        match &self.get(ty_id) {
            TyKind::Adt(def_id, _) => Some(*def_id),
            TyKind::Int(_)
            | TyKind::Float(_)
            | TyKind::Bool
            | TyKind::Char
            | TyKind::String
            | TyKind::Unit
            | TyKind::Never => self.ty_def_map.get(&ty_id).cloned(),

            _ => None,
        }
    }

    pub fn expect_ty_def_id(&self, ty_id: TyId) -> DefId {
        self.get_ty_def_id(ty_id)
            .unwrap_or_else(|| panic!("expected a def id attached to ty id {ty_id:?}"))
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

            BuiltinTyKind::Infer => unreachable!(),
        }
    }

    pub fn make_unit_ty(&mut self) -> TyId {
        self.intern(TyKind::Unit)
    }

    pub fn make_never_ty(&mut self) -> TyId {
        self.intern(TyKind::Never)
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

    pub fn make_fn_ty(
        &mut self,
        arguments: Arc<[GenericArg]>,
        def_id: Option<DefId>,
        params: Arc<[TyId]>,
        ret_ty: TyId,
    ) -> TyId {
        self.intern(TyKind::Fn(
            Box::new(FnTy {
                def_id,
                params,
                ret_ty,
            }),
            arguments,
        ))
    }

    pub fn make_adt_ty(&mut self, def_id: DefId, arguments: Arc<[GenericArg]>) -> TyId {
        self.intern(TyKind::Adt(def_id, arguments))
    }

    pub fn make_inferred_ty(&mut self, span: Span, kind: InferKind) -> TyId {
        let var_id = self.fresh_var(span);
        self.intern(TyKind::Infer(var_id, kind))
    }

    pub fn make_param_ty(&mut self, def_id: DefId, depth: u32, idx: u32) -> TyId {
        self.intern(TyKind::Param(ParamTy::new(def_id, depth, idx)))
    }

    pub fn unresolved_infer(&mut self, ty_id: TyId, tys: &mut Vec<TyId>) {
        let ty_id = self.resolve_ty(ty_id);
        match self.get(ty_id) {
            TyKind::Infer(var_id, _) => tys.push(ty_id),

            TyKind::Array(inner) => self.unresolved_infer(*inner, tys),
            TyKind::Slice(inner) => self.unresolved_infer(*inner, tys),
            TyKind::Tuple(inner_tys) => inner_tys
                .clone()
                .iter()
                .for_each(|ty_id| self.unresolved_infer(*ty_id, tys)),

            TyKind::Ref(inner, _) => self.unresolved_infer(*inner, tys),

            TyKind::Fn(fn_ty, args) => {
                let (params, ret_ty) = (fn_ty.params.clone(), fn_ty.ret_ty);
                args.clone().iter().for_each(|arg| match &arg {
                    GenericArg::Ty(ty_id) => self.unresolved_infer(*ty_id, tys),
                });

                params
                    .clone()
                    .iter()
                    .for_each(|param| self.unresolved_infer(*param, tys));

                self.unresolved_infer(ret_ty, tys);
            }

            TyKind::Adt(_, args) => args.clone().iter().for_each(|arg| match &arg {
                GenericArg::Ty(ty_id) => self.unresolved_infer(*ty_id, tys),
            }),

            _ => {}
        }
    }

    pub fn is_inferred(&self, ty_id: TyId) -> bool {
        match self.get(ty_id) {
            TyKind::Infer(_, InferKind::Any) => true,

            TyKind::Array(inner) => self.is_inferred(*inner),
            TyKind::Slice(inner) => self.is_inferred(*inner),
            TyKind::Tuple(tys) => tys.iter().any(|ty_id| self.is_inferred(*ty_id)),

            TyKind::Ref(inner, _) => self.is_inferred(*inner),

            TyKind::Fn(fn_ty, args) => {
                args.iter().any(|arg| match &arg {
                    GenericArg::Ty(ty_id) => self.is_inferred(*ty_id),
                }) || fn_ty.params.iter().any(|param| self.is_inferred(*param))
                    || self.is_inferred(fn_ty.ret_ty)
            }

            TyKind::Adt(_, args) => args.iter().any(|arg| match &arg {
                GenericArg::Ty(ty_id) => self.is_inferred(*ty_id),
            }),

            _ => false,
        }
    }

    // TODO: TEMP
    pub fn deref(&self, ty_id: TyId) -> TyId {
        match &self.get(ty_id) {
            TyKind::Ref(ty_id, _) => *ty_id,

            // TODO: check protocols of the type if it has a deref proto
            TyKind::Adt(_, _) => ty_id,

            _ => ty_id,
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
