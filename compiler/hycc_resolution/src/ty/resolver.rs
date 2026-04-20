use std::collections::HashMap;

use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirId,
    def::{
        BuiltinIntTy, BuiltinKind, BuiltinTyKind, DefId, DefKind, DefSpace, Definition,
        DefinitionTable,
    },
};
use hycc_ty::{
    context::{TyCtx, TyId},
    ty::IntTy,
};

use crate::{ResolveResult, diag::ResolverDiagCtx};

#[derive(Debug)]
pub struct TyResolver<'d> {
    pub dctx: ResolverDiagCtx,

    pub tctx: TyCtx,
    pub(crate) definitions: &'d DefinitionTable,
}

impl<'d> TyResolver<'d> {
    pub fn new(definitions: &'d DefinitionTable) -> Self {
        Self {
            dctx: ResolverDiagCtx::new(),
            tctx: TyCtx::new(),
            definitions,
        }
    }

    pub fn resolve(&mut self, resolved: &HashMap<HirId, DefId>) {
        for (hir_id, def_id) in resolved {
            let def = self.definitions.get(*def_id);
            if def.kind.space() != DefSpace::Type {
                continue;
            }

            match self.resolve_def(&def) {
                Ok(ty_id) => self.tctx.attach_to_hir(*hir_id, ty_id),
                Err(Some(diag)) => {
                    self.dctx.add(diag);
                }
                _ => {}
            };
        }

        // TODO: iterate over the definitions to resolve types of definitions;
    }

    pub(crate) fn resolve_def(&mut self, def: &Definition) -> ResolveResult<TyId> {
        match &def.kind {
            DefKind::Builtin(kind) => match kind {
                BuiltinKind::Ty(kind) => self.resolve_builtin_ty(&kind),

                #[allow(unreachable_patterns)]
                _ => unreachable!(),
            },

            DefKind::Petal => todo!("resolve petal"),
            DefKind::Struct(strct) => todo!("resolve struct {strct:?}"),

            _ => unreachable!(),
        }
    }

    pub(crate) fn resolve_builtin_ty(&mut self, kind: &BuiltinTyKind) -> ResolveResult<TyId> {
        Ok(match kind {
            BuiltinTyKind::Int(kind) => match kind {
                BuiltinIntTy::Fixed(width, signed) => {
                    self.tctx.make_int_ty(IntTy::Fixed(*width, *signed))
                }
                BuiltinIntTy::Size(signed) => self.tctx.make_int_ty(IntTy::Size(*signed)),
            },

            _ => todo!("other builtin type kinds"),
        })
    }
}
