use hycc_diagnostic::diagnostic::{DiagCtx, Diagnostics, FromResultEmitter};
use hycc_hir::{
    block::HirBlock,
    def::{
        Binding, DefAccessibility, DefId, DefKind, DefNodeResolution, DefResKind, DefResolution,
        DefSpace, Definition, DefinitionTable,
    },
    expr::{
        HirArrayExpr, HirExpr, HirExprKind, HirFnCall, HirIfExpr, HirMethodCall, HirStructExpr,
        HirTupleExpr,
    },
    item::{HirItem, HirItemKind, HirPetal, HirPetalKind, HirReferTarget, HirReferTargetKind},
    path::{HirIdent, HirIdentArgument, HirPath},
    scope::{Scope, ScopeId},
    stmt::{HirStmt, HirStmtKind},
    ty::{HirTy, HirTyKind},
};
use hycc_symbol::{Symbol, SymbolInterner};
use hycc_ty::context::TyCtx;
use hycc_util::{bug, ternary};

use crate::{
    collector::Collector,
    diag::{ResolveResult, ResolverDiag, ResolverDiagCtx, ResolverDiagErrorKind},
};

#[derive(Debug)]
pub struct Resolver<'r> {
    pub dctx: ResolverDiagCtx<'r>,
    pub collector: Collector<'r>,

    pub(crate) expected_space: Option<DefSpace>,
}

impl<'r> Resolver<'r> {
    pub fn new(dctx: &'r mut DiagCtx, collector: Collector<'r>) -> Self {
        Self {
            dctx: ResolverDiagCtx::new(dctx),
            collector,

            expected_space: None,
        }
    }

    pub fn get_binding(
        &self,
        space: Option<DefSpace>,
        name: Symbol,
    ) -> Option<(&Binding, ScopeId)> {
        let scope_id = self
            .collector
            .petal_ctx
            .top()?
            .scope_id(&self.collector.scope_ctx);

        self.collector
            .scope_ctx
            .get_def_until_scope(space, name, scope_id)
            .map(|(binding, _)| (binding, scope_id))
    }

    pub fn get_def_id(&self, space: Option<DefSpace>, name: Symbol) -> Option<(DefId, ScopeId)> {
        self.get_binding(space, name)
            .map(|(binding, scope_id)| (binding.def_id, scope_id))
    }

    pub fn get_def(&self, space: Option<DefSpace>, name: Symbol) -> Option<&Definition> {
        let (def_id, _) = self.get_def_id(space, name)?;
        Some(self.collector.definitions.get(def_id))
    }

    pub fn expect_space<F, U>(&mut self, space: DefSpace, mut handler: F) -> U
    where
        F: FnMut(&mut Self) -> U,
    {
        let prev_space = self.expected_space;
        self.expected_space = Some(space);

        let data = handler(self);

        self.expected_space = prev_space;
        data
    }

    pub fn enter_scope<F, U>(&mut self, scope_id: ScopeId, mut handler: F) -> U
    where
        F: FnMut(&mut Self) -> U,
    {
        let pushed = self.collector.scope_ctx.push_id(scope_id);

        let data = handler(self);
        if pushed {
            self.collector.scope_ctx.pop();
        }

        data
    }

    pub fn resolve(&mut self, tree: &HirItem) {
        self.collector.collect(&tree, &mut self.dctx);
        self.resolve_petal(&tree).emit(&mut self.dctx);
    }

    pub(crate) fn resolve_item(&mut self, item: &HirItem) -> ResolveResult {
        match &item.kind {
            HirItemKind::Refer(_) => self.resolve_refer(&item),
            HirItemKind::Petal(_) => self.resolve_petal(&item),
            HirItemKind::Proto(_) => todo!("resolve proto"),
            HirItemKind::Extend(_) => self.resolve_extend(&item),
            HirItemKind::Struct(_) => self.resolve_struct(&item),
            HirItemKind::Fn(_) => self.resolve_fn(&item),
            HirItemKind::VarDecl(_) => self.resolve_var_decl(&item),
        }
    }

    pub(crate) fn resolve_refer(&mut self, refer_item: &HirItem) -> ResolveResult {
        let HirItemKind::Refer(refer) = &refer_item.kind else {
            unreachable!();
        };

        self.resolve_refer_target(&refer.target, refer_item.accessibility, None)
            .emit(&mut self.dctx);

        Ok(())
    }

    pub(crate) fn resolve_refer_target(
        &mut self,
        target: &HirReferTarget,
        accessibility: DefAccessibility,
        mut resolution: Option<DefResolution>,
    ) -> ResolveResult {
        match &target.kind {
            HirReferTargetKind::Child(alias) => {
                let target = target.symbol;
                let res = self.resolve_ident(&target, resolution)?.unwrap();

                // TODO: improve
                let DefResolution::Petal(def_id) = res else {
                    todo!("throw error: cannot `refer` to non-petal definitions")
                };

                let sym = target.ident.ident;
                let actual = self.collector.definitions.get(def_id);

                self.collector.scope_ctx.top_mut().define(
                    actual.kind.space(),
                    alias.unwrap_or(sym),
                    Binding::new(def_id, accessibility),
                );
            }

            HirReferTargetKind::Parent(children) => {
                resolution = self.expect_space(
                    DefSpace::Type,
                    |s| -> ResolveResult<Option<DefResolution>> {
                        s.resolve_ident(&target.symbol, resolution)
                    },
                )?;

                for child in children {
                    self.resolve_refer_target(&child, accessibility, resolution)
                        .emit(&mut self.dctx);
                }
            }
        }

        Ok(())
    }

    pub(crate) fn resolve_petal(&mut self, item: &HirItem) -> ResolveResult {
        let petal = item.expect_petal();
        let petals = petal.path().map_or_else(
            || vec![self.collector.petal_ctx.root_petal_id()],
            |path| {
                path.segments
                    .iter()
                    .map(|segment| {
                        let def_id = self.collector.definitions.expect_def_id(segment.id);
                        self.collector.petal_ctx.expect_def_petal_id(def_id)
                    })
                    .collect()
            },
        );

        for petal_id in &petals {
            self.collector.petal_ctx.push(*petal_id);
            self.collector.scope_ctx.push_id(
                self.collector
                    .petal_ctx
                    .expect(*petal_id)
                    .scope_id(&self.collector.scope_ctx),
            );
        }

        for item in &petal.items {
            self.resolve_item(&item).emit(&mut self.dctx);
        }

        for _ in 0..petals.len() {
            self.collector.petal_ctx.pop();
            self.collector.scope_ctx.pop();
        }

        Ok(())
    }

    pub(crate) fn resolve_extend(&mut self, item: &HirItem) -> ResolveResult {
        let extend = item.expect_extend();

        let scope_id = self.collector.scope_ctx.expect_hir_scope_id(item.id);
        self.enter_scope(scope_id, |s| {
            if let Some(generic_params) = &extend.generic_params {
                for generic_param in &generic_params.list {
                    for proto_req in &generic_param.proto_reqs {
                        s.expect_space(DefSpace::Type, |s| s.resolve_path(proto_req))
                            .emit(&mut s.dctx);
                    }
                }
            }

            // Resolve extension target
            s.resolve_ty(&extend.target).emit(&mut s.dctx);

            // Resolve extension items
            for item in &extend.items {
                s.resolve_item(&item).emit(&mut s.dctx);
            }
        });

        // if let HirTyKind::Path(path) = &extend.target.kind {
        //     let def_id = self.collector.definitions.expect_def_id(path.id);
        //     let def = self.collector.definitions.get(def_id);
        //     let def_petal = def.petal;
        //     let target = ExtTargetKind::Nominal(ternary!(
        //         matches!(def.kind, DefKind::GenericParam(_)),
        //         ExtNominalTargetKind::Blanket,
        //         ExtNominalTargetKind::Def(def_id)
        //     ));

        //     // let ext_id = self.collector.tctx.ext_table.attach(
        //     //     target,
        //     //     Extension::new(
        //     //         item.id,
        //     //         None,
        //     //         std::mem::take(
        //     //             self.collector
        //     //                 .scope_ctx
        //     //                 .expect_hir_mut_scope(extend.target.id),
        //     //         )
        //     //         .all()
        //     //         .into_iter()
        //     //         .map(|(key, binding)| {
        //     //             let item_def = self.collector.definitions.get_mut(binding.def_id);
        //     //             item_def.petal = def_petal;
        //     //             (key, binding)
        //     //         })
        //     //         .collect::<HashMap<_, _>>(),
        //     //     ),
        //     // );

        //     self.collector
        //         .tctx
        //         .ext_table
        //         .attach_hir_ext_id(item.id, ext_id);
        // }

        Ok(())
    }

    pub(crate) fn resolve_struct(&mut self, item: &HirItem) -> ResolveResult {
        let strct = item.expect_struct();
        let def_id = self.collector.definitions.expect_def_id(item.id);
        let scope_id = self.collector.scope_ctx.expect_def_scope_id(def_id);

        self.enter_scope(scope_id, |s| {
            if let Some(generic_params) = &strct.generic_params {
                for generic_param in &generic_params.list {
                    for proto_req in &generic_param.proto_reqs {
                        s.expect_space(DefSpace::Type, |s| s.resolve_path(proto_req))
                            .emit(&mut s.dctx);
                    }
                }
            }

            for field in &strct.fields.list {
                s.resolve_ty(&field.ty).emit(&mut s.dctx);
            }
        });

        Ok(())
    }

    pub(crate) fn resolve_fn(&mut self, item: &HirItem) -> ResolveResult {
        let func = item.expect_fn();

        let def_id = self.collector.definitions.expect_def_id(item.id);
        let scope_id = self.collector.scope_ctx.expect_def_scope_id(def_id);

        self.enter_scope(scope_id, |s| {
            if let Some(generic_params) = &func.sig.generic_params {
                for generic_param in &generic_params.list {
                    for proto_req in &generic_param.proto_reqs {
                        s.expect_space(DefSpace::Type, |s| s.resolve_path(proto_req))
                            .emit(&mut s.dctx);
                    }
                }
            }

            for param in &func.sig.params.list {
                s.resolve_ty(&param.ty).emit(&mut s.dctx);
            }

            if let Some(ret_ty) = &func.sig.ret_ty {
                s.resolve_ty(&ret_ty).emit(&mut s.dctx);
            }

            s.resolve_block(&func.body).emit(&mut s.dctx);
        });

        Ok(())
    }

    pub(crate) fn resolve_var_decl(&mut self, item: &HirItem) -> ResolveResult {
        let decl = item.expect_var();

        if !item.is_top_level() {
            self.collector
                .collect_var(item, &mut self.dctx)
                .emit(&mut self.dctx);
        }

        if let Some(ty) = decl.ty {
            self.resolve_ty(&ty).emit(&mut self.dctx);
        }

        if let Some(expr) = decl.val {
            self.resolve_expr(&expr).emit(&mut self.dctx);
        }

        Ok(())
    }

    pub(crate) fn resolve_block(&mut self, block: &HirBlock) -> ResolveResult {
        let scope_id = self.collector.scope_ctx.expect_hir_scope_id(block.id);
        self.enter_scope(scope_id, |s| {
            for stmt in &block.stmts {
                s.resolve_stmt(&stmt).emit(&mut s.dctx);
            }
        });

        Ok(())
    }

    pub(crate) fn resolve_stmt(&mut self, stmt: &HirStmt) -> ResolveResult {
        match &stmt.kind {
            HirStmtKind::Ret(ret) => ternary!(
                ret.value.is_some(),
                self.resolve_expr(&ret.value.unwrap()),
                Ok(())
            ),
            HirStmtKind::Pass(pass) => {
                ternary!(
                    pass.value.is_some(),
                    self.resolve_expr(&pass.value.unwrap()),
                    Ok(())
                )
            }

            HirStmtKind::Item(item) => self.resolve_item(&item),
            HirStmtKind::Expr(expr) => self.resolve_expr(&expr),
        }
    }

    pub(crate) fn resolve_path(&mut self, path: &HirPath) -> ResolveResult {
        let space = self
            .expected_space
            .expect("expected definition space must exist");

        let n = path.segments.len();
        let (mut resolution, mut resolved) = (None, 0_usize);

        for (i, segment) in path.segments.iter().enumerate() {
            let is_last = i == n - 1;

            let res = (self.expect_space(
                ternary!(is_last, space, DefSpace::Type),
                |s| -> ResolveResult<Option<DefResolution>> {
                    s.resolve_ident(&segment, resolution)
                },
            ))?;

            if let Some(res) = res {
                resolution.replace(res);
                resolved += 1;
            }

            if is_last {
                self.collector
                    .definitions
                    .get_def_id(segment.id)
                    .map(|def_id| self.collector.definitions.define_id_hir(path.id, def_id));
            }
        }

        self.collector.definitions.attach_res(
            path.id,
            DefNodeResolution {
                base: resolution.unwrap(),
                unresolved: n - resolved,
            },
        );

        Ok(())
    }

    // std::vec::Vec<i32>::ElementType
    // std - (Default) petal (Resolution::Petal(DefId))
    // vec - (Petal-based (Scope)) petal (Resolution::Petal(DefId))
    // Vec<i32> - (Petal-based (Scope)) type (Resolution::Ty(HirId))
    // ElementType - (Type-based (`extend`-based lookup)) type (Resolution::Ty(HirId))

    pub(crate) fn resolve_ident(
        &mut self,
        ident: &HirIdent,
        resolution: Option<DefResolution>,
    ) -> ResolveResult<Option<DefResolution>> {
        let name = ident.ident.ident;
        if let Some(arguments) = &ident.arguments {
            for argument in &arguments.data {
                let res = match &argument {
                    HirIdentArgument::Expr(expr) => self.resolve_expr(&expr),
                    HirIdentArgument::Ty(ty) => self.resolve_ty(&ty),
                };

                res.emit(&mut self.dctx);
            }
        }

        if !matches!(&resolution, Some(DefResolution::Petal(_)) | None) {
            return Ok(None);
        }

        let Some(&Binding { def_id, .. }) = resolution.map_or_else(
            || {
                self.get_binding(self.expected_space, name)
                    .map(|(binding, _)| binding)
            },
            |res| match &res {
                DefResolution::Petal(def_id) => self
                    .collector
                    .scope_ctx
                    .expect_def_scope(*def_id)
                    .get(self.expected_space, name),
                _ => bug!(
                    "cannot resolve identifiers from the given resolution {:?}",
                    res
                ),
            },
        ) else {
            return Err(ResolverDiag::error(
                ident.span,
                ResolverDiagErrorKind::UnrecognizedSymbol(name, self.expected_space),
            ));
        };

        self.collector.definitions.define_id_hir(ident.id, def_id);

        let def = self.collector.definitions.get(def_id);
        let res_kind = def.kind.res_kind();

        if !self.collector.petal_ctx.accessible(&def) {
            Err(ResolverDiag::error(
                ident.span,
                ResolverDiagErrorKind::InaccessibleSymbol(name),
            ))?
        }

        Ok(Some(match &res_kind {
            DefResKind::Petal => DefResolution::Petal(def_id),
            DefResKind::Ty => DefResolution::Ty(def_id),
            DefResKind::Value => DefResolution::Value(def_id),
        }))
    }

    pub(crate) fn resolve_ty(&mut self, ty: &HirTy) -> ResolveResult {
        self.expect_space(DefSpace::Type, |s| match &ty.kind {
            HirTyKind::Unit(..) => Ok(()),

            HirTyKind::Path(path) => s.resolve_path(path),
            HirTyKind::Ref(reference) => s.resolve_ty(&reference.ty),

            HirTyKind::Array(array) => {
                s.resolve_expr(&array.size).emit(&mut s.dctx);
                s.resolve_ty(&array.ty)
            }

            HirTyKind::Slice(slice) => s.resolve_ty(&slice.ty),

            HirTyKind::Tuple(tup) => {
                for element in &tup.data {
                    s.resolve_ty(&element).emit(&mut s.dctx);
                }

                Ok(())
            }

            HirTyKind::Fn(func) => {
                for param in &func.params {
                    s.resolve_ty(&param).emit(&mut s.dctx);
                }

                if let Some(ret_ty) = func.ret_ty {
                    s.resolve_ty(&ret_ty).emit(&mut s.dctx);
                }

                Ok(())
            }
        })
    }

    pub(crate) fn resolve_expr(&mut self, expr: &HirExpr) -> ResolveResult {
        self.expect_space(DefSpace::Value, |s| match &expr.kind {
            HirExprKind::Path(path) => s.resolve_path(&path),
            HirExprKind::RefExpr(reference) => s.resolve_expr(&reference.expr),

            HirExprKind::Literal(_) => Ok(()),

            HirExprKind::Binary(_, left, right) => s.resolve_binary_expr(&left, &right),
            HirExprKind::Unary(unary) => s.resolve_expr(unary.expr()),

            HirExprKind::Assign(assignee, expr) => s.resolve_assign_expr(&assignee, &expr),

            HirExprKind::Block(block) => s.resolve_block(&block),

            HirExprKind::Array(array) => s.resolve_array_expr(&array),
            HirExprKind::Tuple(tup) => s.resolve_tuple_expr(&tup),
            HirExprKind::Struct(strct) => s.resolve_struct_expr(&strct),
            HirExprKind::AnonFn(anfn) => s.resolve_anon_fn_expr(&expr),

            HirExprKind::FnCall(call) => s.resolve_fn_call_expr(&call),
            HirExprKind::FieldAccess(access) => s.resolve_expr(access.leading),
            HirExprKind::MethodCall(call) => s.resolve_method_call_expr(&call),

            HirExprKind::If(ite) => s.resolve_if_expr(&ite),
        })
    }

    pub(crate) fn resolve_binary_expr(&mut self, left: &HirExpr, right: &HirExpr) -> ResolveResult {
        self.resolve_expr(&left).emit(&mut self.dctx);
        self.resolve_expr(&right)
    }

    pub(crate) fn resolve_assign_expr(
        &mut self,
        assignee: &HirExpr,
        expr: &HirExpr,
    ) -> ResolveResult {
        if let Err(diag) = self.resolve_expr(&assignee) {
            self.dctx.add(diag);
        }

        self.resolve_expr(&expr)
    }

    pub(crate) fn resolve_array_expr(&mut self, array: &HirArrayExpr) -> ResolveResult {
        for expr in &array.elements {
            if let Err(diag) = self.resolve_expr(&expr) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }

    pub(crate) fn resolve_tuple_expr(&mut self, tup: &HirTupleExpr) -> ResolveResult {
        for el in &tup.elements {
            if let Err(diag) = self.resolve_expr(&el) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }

    pub(crate) fn resolve_struct_expr(&mut self, strct: &HirStructExpr) -> ResolveResult {
        self.expect_space(DefSpace::Type, |s| {
            if let Err(diag) = s.resolve_path(&strct.path) {
                s.dctx.add(diag);
            }
        });

        Ok(for field in &strct.fields {
            if let Err(diag) = self.resolve_expr(&field.val) {
                self.dctx.add(diag);
            }
        })
    }

    pub(crate) fn resolve_anon_fn_expr(&mut self, anfn_expr: &HirExpr) -> ResolveResult {
        let HirExprKind::AnonFn(anfn) = &anfn_expr.kind else {
            unreachable!()
        };

        let scope_id = self.collector.scope_ctx.expect_hir_scope_id(anfn_expr.id);
        self.enter_scope(scope_id, |s| {
            for param in &anfn.params.list {
                let Some(p_ty) = param.ty else {
                    continue;
                };

                if let Err(diag) = s.resolve_ty(&p_ty) {
                    s.dctx.add(diag);
                }
            }

            if let Some(ret_ty) = &anfn.ret_ty {
                if let Err(diag) = s.resolve_ty(&ret_ty) {
                    s.dctx.add(diag);
                }
            }

            s.resolve_block(&anfn.body)
        });

        Ok(())
    }

    pub(crate) fn resolve_fn_call_expr(&mut self, call: &HirFnCall) -> ResolveResult {
        if let Err(diag) = self.resolve_expr(&call.callee) {
            self.dctx.add(diag);
        }

        for argument in &call.arguments.data {
            if let Err(diag) = self.resolve_expr(&argument) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }

    pub(crate) fn resolve_method_call_expr(&mut self, call: &HirMethodCall) -> ResolveResult {
        if let Err(diag) = self.resolve_expr(&call.receiver) {
            self.dctx.add(diag);
        }

        if let Some(arguments) = &call.callee.arguments {
            for argument in &arguments.data {
                let res = match &argument {
                    HirIdentArgument::Expr(expr) => {
                        self.expect_space(DefSpace::Value, |s| s.resolve_expr(&expr))
                    }

                    HirIdentArgument::Ty(ty) => {
                        self.expect_space(DefSpace::Type, |s| s.resolve_ty(&ty))
                    }
                };

                if let Err(diag) = res {
                    self.dctx.add(diag);
                }
            }
        }

        for argument in &call.arguments.data {
            if let Err(diag) = self.resolve_expr(&argument) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }

    pub(crate) fn resolve_if_expr(&mut self, ite: &HirIfExpr) -> ResolveResult {
        if let Err(diag) = self.resolve_expr(&ite.cond) {
            self.dctx.add(diag);
        }

        if let Err(diag) = self.resolve_block(&ite.consequent) {
            self.dctx.add(diag);
        }

        ite.alternate.as_ref().map(|alt| {
            if let Err(diag) = self.resolve_block(&alt) {
                self.dctx.add(diag);
            }
        });

        Ok(())
    }
}
