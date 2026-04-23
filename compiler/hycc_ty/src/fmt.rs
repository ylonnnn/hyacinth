use hycc_hir::def::DefinitionTable;
use hycc_symbol::SymbolInterner;
use hycc_util::ternary;

use crate::ty::{InferKind, IntTy, TyKind};

#[derive(Debug)]
pub struct TyFormatter<'d, 'i> {
    definitions: &'d DefinitionTable,
    interner: &'i SymbolInterner,
}

impl<'d, 'i> TyFormatter<'d, 'i> {
    pub fn new(definitions: &'d DefinitionTable, interner: &'i SymbolInterner) -> Self {
        Self {
            definitions,
            interner,
        }
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

            TyKind::Adt(def_id) => {
                // TODO: add generic arguments
                let def = self.definitions.get(*def_id);
                format!("{}", self.interner.get(def.name))
            }

            TyKind::Infer(_, kind) => match kind {
                InferKind::Any => String::from("_"),
                InferKind::Int => String::from("integer"),
                InferKind::Float => String::from("float"),
            },

            TyKind::Param(def_id) => {
                let def = self.definitions.get(*def_id);
                format!("{}", self.interner.get(def.name))
            }
        }
    }
}
