use hycc_diagnostic::DiagnosticContext;
use hycc_hir::item::{HirFn, HirItem, HirItemKind, HirPetal, HirStruct, HirVarDecl};

use crate::{ResolveResult, ty::resolver::TyResolver};

impl<'d, 'r> TyResolver<'d, 'r> {
    pub(crate) fn resolve_item(&mut self, item: &HirItem) -> ResolveResult {
        match &item.kind {
            HirItemKind::Petal(petal) => self.resolve_petal(&petal),
            HirItemKind::Struct(strct) => self.resolve_struct(&strct),
            HirItemKind::Fn(func) => self.resolve_fn(&func),
            HirItemKind::VarDecl(decl) => self.resolve_var_decl(&decl),
        }
    }

    pub(crate) fn resolve_petal(&mut self, petal: &HirPetal) -> ResolveResult {
        for item in &petal.items {
            if let Err(Some(diag)) = self.resolve_item(&item) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }

    pub(crate) fn resolve_struct(&mut self, strct: &HirStruct) -> ResolveResult {
        for field in &strct.fields.list {
            if let Err(Some(diag)) = self.resolve_ty(&field.ty) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }

    pub(crate) fn resolve_fn(&mut self, func: &HirFn) -> ResolveResult {
        for param in &func.params.list {
            if let Err(Some(diag)) = self.resolve_ty(&param.ty) {
                self.dctx.add(diag);
            }
        }

        if let Err(Some(diag)) = self.resolve_block(&func.body) {
            self.dctx.add(diag);
        }

        Ok(())
    }

    pub(crate) fn resolve_var_decl(&mut self, decl: &HirVarDecl) -> ResolveResult {
        if let Some(ty) = decl.ty {
            if let Err(Some(diag)) = self.resolve_ty(&ty) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }
}
