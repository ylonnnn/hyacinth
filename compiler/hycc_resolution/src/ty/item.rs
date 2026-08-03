use std::{collections::HashMap, sync::Arc};

use hycc_diagnostic::DiagnosticContext;
use hycc_hir::item::{HirExtend, HirItem, HirItemKind, HirPetal, HirStruct};
use hycc_ty::{
    extension::Extension,
    ty::{InferKind, Ty},
};

use crate::{ResolveResult, ty::resolver::TyResolver};

impl<'t, 'd, 's> TyResolver<'t, 'd, 's> {
    pub(crate) fn resolve_item(&mut self, item: &HirItem) -> ResolveResult {
        match &item.kind {
            HirItemKind::Refer(_) => Ok(()),
            HirItemKind::Petal(petal) => self.resolve_petal(&petal),
            HirItemKind::Proto(_) => todo!("(ty) resolve proto"),
            HirItemKind::Extend(_) => self.resolve_extend(&item),
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

    pub(crate) fn resolve_extend(&mut self, extend_item: &HirItem) -> ResolveResult {
        let HirItemKind::Extend(extend) = &extend_item.kind else {
            unreachable!()
        };

        // Resolve the target type
        let target_ty_id = self.resolve_path(&extend.target)?;

        let target_def_id = self.definitions.expect_def_id(extend.target.id);
        let ty_scope = self.scope_ctx.expect_def_scope(target_def_id);

        self.tctx.ext_table.attach(
            target_def_id,
            Extension::new(
                target_ty_id,
                extend
                    .items
                    .iter()
                    .filter_map(|item| {
                        let def = self.definitions.get_def(item.id).unwrap();
                        let (space, name) = (def.kind.space(), def.name);

                        ty_scope
                            .get(Some(space), name)
                            .cloned()
                            .map(|binding| ((space, name), binding))
                    })
                    .collect::<HashMap<_, _>>(),
            ),
        );

        // Resolve the items of the extension
        for item in &extend.items {
            if let Err(Some(diag)) = self.resolve_item(&item) {
                self.dctx.add(diag);
            }
        }

        Ok(())
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

        let def_id = self.definitions.expect_def_id(fn_item.id);
        let mut params = Vec::new();

        // func.sig.generic_params;

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

        let fn_ty_id = self.tctx.make_fn_ty(
            /* TODO */ Arc::new([]),
            Some(def_id),
            params.into(),
            ret_ty,
        );
        let fn_ty = Ty::new(fn_ty_id, fn_item.span);

        self.tctx.attach_to_hir(fn_item.id, fn_ty.clone());
        self.tctx
            .attach_to_def(self.definitions.get_def_id(fn_item.id).unwrap(), fn_ty);

        if let Err(Some(diag)) = self.resolve_block(&func.body) {
            self.dctx.add(diag);
        }

        Ok(())
    }

    pub(crate) fn resolve_var_decl(&mut self, var_decl: &HirItem) -> ResolveResult {
        let HirItemKind::VarDecl(decl) = &var_decl.kind else {
            unreachable!()
        };

        let (ty_id, span) = match &decl.ty {
            Some(ty) => (self.resolve_ty(ty)?, ty.span),
            None => (self.tctx.make_inferred_ty(InferKind::Any), decl.span),
        };

        self.tctx.attach_to_hir(var_decl.id, Ty::new(ty_id, span));

        if let Some(expr) = decl.val {
            if let Err(Some(diag)) = self.resolve_expr(&expr) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }
}
