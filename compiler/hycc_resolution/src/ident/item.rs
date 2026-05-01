use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    def::DefSpace,
    item::{HirItem, HirItemKind, HirPetalKind, HirStruct},
};
use hycc_util::bug;

use crate::{ResolveResult, ident::resolver::Resolver};

impl<'c> Resolver<'c> {
    pub(crate) fn resolve_item(&mut self, item: &HirItem) -> ResolveResult {
        match &item.kind {
            HirItemKind::Petal(_) => self.resolve_petal(&item),
            HirItemKind::Struct(_) => self.resolve_struct(&item),
            HirItemKind::Fn(_) => self.resolve_fn(&item),
            HirItemKind::VarDecl(_) => self.resolve_var_decl(&item),
        }
    }

    pub(crate) fn resolve_petal(&mut self, petal_item: &HirItem) -> ResolveResult {
        let HirItemKind::Petal(petal) = &petal_item.kind else {
            unreachable!()
        };

        if matches!(petal.kind, HirPetalKind::Root) {
            panic!("root petals cannot be collected!")
        }

        if let Err(Some(diag)) = self.collector.collect_petal(&petal_item) {
            self.collector.dctx.add(diag);
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

            let Some(scope_id) = self.collector.scope_ctx.get_id_from_def(def_id) else {
                bug!("no scope for petal def: {:?}", def_id)
            };

            self.collector.scope_ctx.push_id(scope_id);
            scopes += 1;
        }

        for item in &petal.items {
            if let Err(Some(diag)) = self.resolve_item(&item) {
                self.dctx.add(diag);
            }
        }

        while scopes > 0 {
            self.collector.scope_ctx.pop();
            scopes -= 1;
        }

        Ok(())
    }

    pub(crate) fn resolve_struct(&mut self, struct_item: &HirItem) -> ResolveResult {
        let HirItemKind::Struct(strct) = &struct_item.kind else {
            unreachable!()
        };

        if let Err(Some(diag)) = self.collector.collect_struct(&struct_item) {
            self.collector.dctx.add(diag);
        }

        for field in &strct.fields.list {
            if let Err(Some(diag)) = self.resolve_ty(&field.ty) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }

    pub(crate) fn resolve_fn(&mut self, fn_item: &HirItem) -> ResolveResult {
        let HirItemKind::Fn(func) = &fn_item.kind else {
            unreachable!()
        };

        if let Err(Some(diag)) = self.collector.collect_fn(&fn_item) {
            self.collector.dctx.add(diag);
        }

        let Some(def_id) = self.get_def_id(DefSpace::Value, func.ident.ident) else {
            bug!("no def_id for ident: {:?}", func.ident.ident)
        };

        let Some(scope_id) = self.collector.scope_ctx.get_id_from_def(def_id) else {
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

    pub(crate) fn resolve_var_decl(&mut self, var_decl: &HirItem) -> ResolveResult {
        let HirItemKind::VarDecl(decl) = &var_decl.kind else {
            unreachable!()
        };

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

        if let Err(Some(diag)) = self.collector.collect_var(var_decl) {
            self.collector.dctx.add(diag);
        }

        Ok(())
    }
}
