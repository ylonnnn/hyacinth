use std::{collections::HashMap, sync::Arc};

use hycc_diagnostic::DiagnosticContext;
use hycc_hir::item::{HirExtend, HirItem, HirItemKind, HirPetal, HirStruct};
use hycc_symbol::Symbol;
use hycc_ty::{
    context::TyId,
    extension::{Extension, ExtensionTarget},
    ty::{GenericArg, InferKind, Ty},
};

use crate::{
    ResolveResult,
    diag::{ResolverDiag, ResolverDiagErrorKind},
    ty::resolver::TyResolver,
};

impl<'t, 'd, 's, 'h> TyResolver<'t, 'd, 's, 'h> {
    pub(crate) fn resolve_item(&mut self, item: &HirItem) -> ResolveResult {
        match &item.kind {
            HirItemKind::Refer(_) => Ok(()),
            HirItemKind::Petal(petal) => self.resolve_petal(&petal),
            HirItemKind::Proto(_) => todo!("(ty) resolve proto"),
            HirItemKind::Extend(_) => self.resolve_extend(&item),
            HirItemKind::Struct(strct) => self.resolve_struct(&strct),
            HirItemKind::Fn(_) => self.resolve_fn(&item).map(|_| ()),
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
        let target_ty_id = self.resolve_ty(&extend.target)?;
        let scope = self.scope_ctx.expect_hir_mut_scope(extend.target.id);

        // TODO: define `Self`
        // dbg!(self.definitions.get_def_id(extend.target.id));
        // self.def_to_ty(def_id, span)

        if let Some(def_id) = self.tctx.get_ty_def_id(target_ty_id) {
            self.tctx
                .ext_table
                .expect_hir_mut_ext(extend_item.id)
                .attach_ty_id(target_ty_id);
        } else {
            let target = ExtensionTarget::Ty(target_ty_id);

            self.tctx.ext_table.attach(
                target,
                Extension::new(
                    extend_item.id,
                    target,
                    Some(target_ty_id),
                    std::mem::take(scope)
                        .all()
                        .into_iter()
                        .collect::<HashMap<_, _>>(),
                ),
            );
        }

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

    pub(crate) fn resolve_fn(&mut self, fn_item: &HirItem) -> ResolveResult<TyId> {
        let HirItemKind::Fn(func) = &fn_item.kind else {
            unreachable!()
        };

        let def_id = self.definitions.expect_def_id(fn_item.id);
        if self.tctx.get_ty_of_def(def_id).is_some() {
            return Ok(self.tctx.get_ty_of_def(def_id).unwrap().id);
        }

        let mut generic_args = Vec::new();
        if let Some(generic_params) = &func.sig.generic_params {
            for generic_param in &generic_params.list {
                let def_id = self.definitions.expect_def_id(generic_param.id);
                let def = self.definitions.get(def_id).kind.expect_generic_param();

                generic_args.push(GenericArg::Ty(self.tctx.make_param_ty(
                    def_id,
                    def.depth(),
                    def.idx(),
                )));
            }
        }

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

        let fn_ty_id =
            self.tctx
                .make_fn_ty(generic_args.into(), Some(def_id), params.into(), ret_ty);
        let fn_ty = Ty::new(fn_ty_id, fn_item.span);

        self.tctx.attach_to_hir(fn_item.id, fn_ty.clone());
        self.tctx
            .attach_to_def(self.definitions.get_def_id(fn_item.id).unwrap(), fn_ty);

        if let Err(Some(diag)) = self.resolve_block(&func.body) {
            self.dctx.add(diag);
        }

        Ok(fn_ty_id)
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
