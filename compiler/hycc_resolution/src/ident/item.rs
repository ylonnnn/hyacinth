use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    def::DefSpace,
    item::{HirFn, HirItem, HirItemKind, HirPetal, HirPetalKind, HirStruct, HirVarDecl},
};
use hycc_util::bug;

use crate::ident::resolver::{ResolveResult, Resolver};

impl<'s, 'd> Resolver<'s, 'd> {
    pub(crate) fn resolve_item(&mut self, item: &HirItem) -> ResolveResult {
        match &item.kind {
            HirItemKind::Petal(petal) => self.resolve_petal(&petal),
            HirItemKind::Struct(strct) => self.resolve_struct(&strct),
            HirItemKind::Fn(func) => self.resolve_fn(&func),
            HirItemKind::VarDecl(decl) => self.resolve_var_decl(&decl),
        }
    }

    pub(crate) fn resolve_petal(&mut self, petal: &HirPetal) -> ResolveResult {
        if matches!(petal.kind, HirPetalKind::Root) {
            panic!("root petals cannot be collected!")
        }

        let path = match &petal.kind {
            HirPetalKind::File(path) | HirPetalKind::Inline(path) => path,
            _ => unreachable!(),
        };

        let mut scopes = 0;
        for segment in &path.segments {
            let Some(def_id) = self.get_def_id(DefSpace::Type, segment.ident.ident) else {
                bug!("no def_id for ident: {:?}", segment.ident.ident)
            };

            let Some(scope_id) = self.scope_ctx.get_id_from_def(def_id) else {
                bug!("no scope for petal def: {:?}", def_id)
            };

            self.scope_ctx.push_id(scope_id);
            scopes += 1;
        }

        for item in &petal.items {
            if let Err(Some(diag)) = self.resolve_item(&item) {
                self.dctx.add(diag);
            }
        }

        while scopes > 0 {
            self.scope_ctx.pop();
            scopes -= 1;
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
        let Some(def_id) = self.get_def_id(DefSpace::Value, func.ident.ident) else {
            bug!("no def_id for ident: {:?}", func.ident.ident)
        };

        let Some(scope_id) = self.scope_ctx.get_id_from_def(def_id) else {
            bug!("no scope for def: {def_id:?}")
        };

        self.enter_scope(scope_id, |s| {
            for param in &func.params.list {
                if let Err(Some(diag)) = s.resolve_ty(&param.ty) {
                    s.dctx.add(diag);
                }
            }

            if let Some(ret_ty) = &func.ret_ty {
                if let Err(Some(diag)) = s.resolve_ty(&ret_ty) {
                    s.dctx.add(diag);
                }
            }

            if let Err(Some(diag)) = s.resolve_block(&func.body) {
                s.dctx.add(diag);
            }
        });

        Ok(())
    }

    pub(crate) fn resolve_var_decl(&mut self, decl: &HirVarDecl) -> ResolveResult {
        if let Some(ty) = decl.ty {
            if let Err(Some(diag)) = self.resolve_ty(&ty) {
                self.dctx.add(diag);
            }
        }

        if let Some(expr) = decl.val {
            if let Err(Some(diag)) = self.resolve_expr(&expr) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }
}
