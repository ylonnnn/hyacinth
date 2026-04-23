use hycc_hir::def::DefinitionTable;
use hycc_symbol::SymbolInterner;
use hycc_util::ternary;

use crate::{
    context::{TyCtx, TyId},
    ty::{InferKind, IntTy, RefMutability, TyKind},
};

#[derive(Debug)]
pub struct TyFormatter<'t, 'd, 'i> {
    pub tctx: &'t TyCtx,
    definitions: &'d DefinitionTable,
    interner: &'i SymbolInterner,
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

            TyKind::Ref(ty_id, mutability) => {
                format!(
                    "&{}{}",
                    ternary!(*mutability == RefMutability::Mutable, "mut ", ""),
                    self.fmt_id(*ty_id)
                )
            }

            TyKind::Adt(def_id) => {
                // TODO: add generic arguments
                let def = self.definitions.get(*def_id);
                format!("{}", self.interner.get(def.name))
            }

            TyKind::Infer(_, kind) => match kind {
                InferKind::Any => String::from("{unknown}"),
                InferKind::Int => String::from("{int}"),
                InferKind::Float => String::from("{float}"),
            },

            TyKind::Param(def_id) => {
                let def = self.definitions.get(*def_id);
                format!("{}", self.interner.get(def.name))
            }
        }
    }
}
