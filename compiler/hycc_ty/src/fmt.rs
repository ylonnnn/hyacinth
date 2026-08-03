use hycc_hir::def::DefinitionTable;
use hycc_symbol::SymbolInterner;
use hycc_util::ternary;

use crate::{
    context::{TyCtx, TyId},
    ty::{GenericArg, InferKind, IntTy, RefMutability, TyKind},
};

#[derive(Debug)]
pub struct TyFormatter<'t, 'd, 'i> {
    pub tctx: &'t TyCtx,
    pub definitions: &'d DefinitionTable,
    pub interner: &'i SymbolInterner,
}

impl<'t, 'd, 'i> TyFormatter<'t, 'd, 'i> {
    pub fn new(
        tctx: &'t TyCtx,
        definitions: &'d DefinitionTable,
        interner: &'i SymbolInterner,
    ) -> Self {
        Self {
            tctx,
            definitions,
            interner,
        }
    }

    pub fn fmt_id(&self, id: TyId) -> String {
        let kind = self.tctx.get(id);
        self.fmt(&kind)
    }

    pub fn fmt(&self, kind: &TyKind) -> String {
        match &kind {
            TyKind::Unit => String::from("()"),
            TyKind::Never => String::from("~"),

            TyKind::Int(data) => match data {
                IntTy::Fixed(width, signed) => format!("{}{width}", ternary!(*signed, "i", "u")),
                IntTy::Size(signed) => format!("{}size", ternary!(*signed, "i", "u")),
            },

            TyKind::Float(width) => format!("f{width}"),
            TyKind::Bool => format!("bool"),
            TyKind::Char => format!("char"),
            TyKind::String => format!("str"),

            TyKind::Array(ty_id) => {
                format!("[<size>]{}", self.fmt_id(*ty_id))
            }

            TyKind::Slice(ty_id) => {
                format!("[]{}", self.fmt_id(*ty_id))
            }

            TyKind::Tuple(tys) => {
                format!(
                    "({})",
                    tys.iter()
                        .map(|ty_id| self.fmt_id(*ty_id))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }

            TyKind::Ref(ty_id, mutability) => {
                format!(
                    "&{}{}",
                    ternary!(*mutability == RefMutability::Mutable, "mut ", ""),
                    self.fmt_id(*ty_id)
                )
            }

            TyKind::Fn(ty, _) => {
                let is_unit = matches!(self.tctx.get(ty.ret_ty), TyKind::Unit);

                format!(
                    "fn({}){}",
                    ty.params
                        .iter()
                        .map(|param| self.fmt_id(*param))
                        .collect::<Vec<_>>()
                        .join(", "),
                    ternary!(
                        is_unit,
                        String::from(""),
                        format!(" -> {}", self.fmt_id(ty.ret_ty))
                    )
                )
            }

            TyKind::Adt(def_id, generic_args) => {
                let def = self.definitions.get(*def_id);
                format!(
                    "{}{}",
                    self.interner.get(def.name),
                    ternary!(
                        generic_args.is_empty(),
                        String::from(""),
                        format!(
                            "<{}>",
                            generic_args
                                .iter()
                                .map(|arg| {
                                    match &arg {
                                        GenericArg::Ty(ty_id) => self.fmt_id(*ty_id),
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    )
                )
            }

            TyKind::Infer(_, kind) => match kind {
                InferKind::Any => String::from("{unknown}"),
                InferKind::Int => String::from("{int}"),
                InferKind::Float => String::from("{float}"),
            },

            TyKind::Param(param) => {
                let def = self.definitions.get(param.def_id);
                format!("{}", self.interner.get(def.name))
            }
        }
    }
}
