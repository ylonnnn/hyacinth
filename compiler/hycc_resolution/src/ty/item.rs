use hycc_diagnostic::DiagnosticContext;
use hycc_hir::item::{HirExtend, HirItem, HirItemKind, HirPetal, HirStruct};
use hycc_ty::ty::Ty;

use crate::{ResolveResult, ty::resolver::TyResolver};

impl<'d> TyResolver<'d> {
    pub(crate) fn resolve_item(&mut self, item: &HirItem) -> ResolveResult {
        match &item.kind {
            HirItemKind::Refer(_) => Ok(()),
            HirItemKind::Petal(petal) => self.resolve_petal(&petal),
            HirItemKind::Proto(_) => todo!("(ty) resolve proto"),
            HirItemKind::Extend(extend) => self.resolve_extend(&extend),
            HirItemKind::Struct(strct) => self.resolve_struct(&strct),
            HirItemKind::Fn(_) => self.resolve_fn(&item),
            HirItemKind::VarDecl(_) => self.resolve_var_decl(&item),
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

    pub(crate) fn resolve_extend(&mut self, extend: &HirExtend) -> ResolveResult {
        extend.target;
        todo!("(ty) resolve extend")
    }

    pub(crate) fn resolve_struct(&mut self, strct: &HirStruct) -> ResolveResult {
        for field in &strct.fields.list {
            if let Err(Some(diag)) = self.resolve_as_non_inferable_ty(&field.ty) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }

    pub(crate) fn resolve_fn(&mut self, fn_item: &HirItem) -> ResolveResult {
        let HirItemKind::Fn(func) = &fn_item.kind else {
            unreachable!()
        };

        let mut params = Vec::new();
        for param in &func.sig.params.list {
            let ty_id = match self.resolve_as_non_inferable_ty(&param.ty) {
                Ok(ty_id) => ty_id,
                Err(diag) => {
                    diag.map(|diag| self.dctx.add(diag));
                    continue;
                }
            };

            params.push(ty_id);
            self.tctx
                .attach_to_hir(param.id, Ty::new(ty_id, param.ty.span));
        }

        let mut ret_ty = self.tctx.make_unit_ty();
        if let Some(r_ty) = &func.sig.ret_ty {
            match self.resolve_as_non_inferable_ty(&r_ty) {
                Ok(ty_id) => ret_ty = ty_id,
                Err(diag) => {
                    diag.map(|diag| self.dctx.add(diag));
                }
            }
        }

        let fn_ty = self.tctx.make_fn_ty(params.into(), ret_ty);
        self.tctx
            .attach_to_hir(fn_item.id, Ty::new(fn_ty, fn_item.span));

        if let Err(Some(diag)) = self.resolve_block(&func.body) {
            self.dctx.add(diag);
        }

        Ok(())
    }

    pub(crate) fn resolve_var_decl(&mut self, var_decl: &HirItem) -> ResolveResult {
        let HirItemKind::VarDecl(decl) = &var_decl.kind else {
            unreachable!()
        };

        if let Some(ty) = decl.ty {
            match self.resolve_ty(&ty) {
                Ok(ty_id) => self
                    .tctx
                    .attach_to_hir(var_decl.id, Ty::new(ty_id, ty.span)),
                Err(Some(diag)) => {
                    self.dctx.add(diag);
                }
                _ => {}
            }
        }

        // Attempt to resolve block expressions
        if let Some(expr) = decl.val {
            if let Err(Some(diag)) = self.resolve_expr(&expr) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }
}
