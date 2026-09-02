use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use hycc_diagnostic::diagnostic::{DiagCtx, Diagnostics, FromResultEmitter};
use hycc_hir::{
    HirId,
    block::HirBlock,
    def::{
        Binding, DefAccessibility, DefId, DefKind, DefNodeResolution, DefResKind, DefResolution,
        DefSpace, Definition, DefinitionTable,
    },
    expr::{
        HirArrayExpr, HirCastExpr, HirExpr, HirExprKind, HirFnCall, HirIfExpr, HirMethodCall,
        HirStructExpr, HirTupleExpr,
    },
    item::{
        HirFnSig, HirIntfItem, HirItem, HirItemKind, HirPetal, HirPetalKind, HirReferTarget,
        HirReferTargetKind, HirVarSig,
    },
    path::{HirIdent, HirIdentArgument, HirPath},
    scope::{Scope, ScopeId},
    stmt::{HirStmt, HirStmtKind},
    ty::{HirTy, HirTyKind},
};
use hycc_symbol::{Symbol, SymbolInterner};
use hycc_ty::ctx::TyCtx;
use hycc_util::{bug, ternary};

use crate::{
    collector::Collector,
    diag::{ResolveResult, ResolverDiag, ResolverDiagCtx, ResolverDiagErrorKind, SymbolKind},
};

#[derive(Debug)]
pub struct Resolver<'r, 'h> {
    pub dctx: ResolverDiagCtx<'r>,
    pub collector: Collector<'r, 'h>,

    pub(crate) expected_space: Option<DefSpace>,
}

impl<'r, 'h> Resolver<'r, 'h> {
    pub fn new(dctx: &'r mut DiagCtx, collector: Collector<'r, 'h>) -> Self {
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
            HirItemKind::Intf(_) => self.resolve_intf(&item),
            HirItemKind::Extend(_) => self.resolve_extend(&item),
            HirItemKind::Struct(_) => self.resolve_struct(&item),
            HirItemKind::FnDecl(sig) => self.resolve_fn_sig(item.id, &sig),
            HirItemKind::Fn(_) => self.resolve_fn(&item),
            HirItemKind::VarDecl(sig) => self.resolve_var_sig(item.id, &sig),
            HirItemKind::VarDef(_) => self.resolve_var_def(&item),
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
                let def_id = self.resolve_ident(&target, resolution)?.unwrap().def_id();

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

        petals.iter().for_each(|petal_id| {
            self.collector.petal_ctx.push(*petal_id);
            self.collector.scope_ctx.push_id(
                self.collector
                    .petal_ctx
                    .expect(*petal_id)
                    .scope_id(&self.collector.scope_ctx),
            );
        });

        petal
            .items
            .iter()
            .for_each(|item| self.resolve_item(&item).emit_discard(&mut self.dctx));

        (0..petals.len()).for_each(|_| {
            self.collector.petal_ctx.pop();
            self.collector.scope_ctx.pop();
        });

        Ok(())
    }

    pub(crate) fn resolve_intf(&mut self, item: &HirItem) -> ResolveResult {
        let intf = item.expect_intf();
        let scope_id = self.collector.scope_ctx.expect_hir_scope_id(item.id);

        self.enter_scope(scope_id, |s| {
            intf.items
                .iter()
                .for_each(|item| s.resolve_intf_item(&item).emit_discard(&mut s.dctx))
        });

        Ok(())
    }

    pub(crate) fn resolve_intf_item(&mut self, item: &HirIntfItem) -> ResolveResult {
        match &item {
            HirIntfItem::Fn(item) | HirIntfItem::Var(item) => self.resolve_item(&item),
        }
    }

    pub(crate) fn resolve_extend(&mut self, item: &HirItem) -> ResolveResult {
        let extend = item.expect_extend();

        let scope_id = self.collector.scope_ctx.expect_hir_scope_id(item.id);
        self.enter_scope(scope_id, |s| {
            extend.generic_params.as_ref().map(|generic_params| {
                generic_params.list.iter().for_each(|generic_param| {
                    generic_param.intf_reqs.iter().for_each(|intf_req| {
                        s.expect_space(DefSpace::Type, |s| s.resolve_path(intf_req))
                            .emit_discard(&mut s.dctx)
                    })
                })
            });

            // Resolve extension target
            s.resolve_ty(&extend.target).emit(&mut s.dctx);

            // Resolve extension interface
            let intf_id = s.expect_space(DefSpace::Type, |s| {
                extend.intf.as_ref().and_then(|intf| {
                    s.resolve_path(intf).emit(&mut s.dctx)?;
                    s.collector
                        .tctx
                        .intf_table
                        .get_hir_intf_id(s.collector.definitions.expect_def(intf.id).hir_id)
                })
            });

            let mut unimplemented = intf_id.map(|intf_id| {
                s.collector
                    .tctx
                    .intf_table
                    .get(intf_id)
                    .items
                    .iter()
                    .map(|(key, item)| (key.clone(), item.kind))
                    .collect::<HashMap<_, _>>()
            });

            // Resolve extension items
            extend.items.iter().for_each(|item| {
                s.resolve_item(&item).emit(&mut s.dctx);

                // No `unimplemented` map implies that there is no
                // interface provided
                let Some(unimp) = unimplemented.as_mut() else {
                    return;
                };

                let def_id = s.collector.definitions.expect_def_id(item.id);
                let def = s.collector.definitions.get(def_id);
                let def_key = (def.kind.space(), def.name);

                if !unimp.contains_key(&def_key) {
                    return s.dctx.error(
                        item.span,
                        ResolverDiagErrorKind::NonIntfAssocItemDefinition {
                            def_id,
                            intf_id: intf_id.unwrap(),
                        },
                    );
                }

                // TODO: check implemented associated item compatibility with their corresponding interface item

                unimp.remove(&def_key);
            });

            intf_id.map(|intf_id| {
                let Some(unimp) = unimplemented else {
                    return;
                };

                let intf = s.collector.tctx.intf_table.get(intf_id);
                let missing_req = unimp
                    .into_iter()
                    .filter_map(|u| ternary!(u.1.is_req(), Some(u.0), None))
                    .collect::<Arc<_>>();
                if !missing_req.is_empty() {
                    s.dctx.error(
                        item.span,
                        ResolverDiagErrorKind::UnimplAssocItem {
                            intf_id,
                            items: missing_req,
                        },
                    );
                }
            })
        });

        Ok(())
    }

    pub(crate) fn resolve_struct(&mut self, item: &HirItem) -> ResolveResult {
        let strct = item.expect_struct();
        let def_id = self.collector.definitions.expect_def_id(item.id);
        let scope_id = self.collector.scope_ctx.expect_def_scope_id(def_id);

        self.enter_scope(scope_id, |s| {
            strct.generic_params.as_ref().map(|generic_params| {
                generic_params.list.iter().for_each(|generic_param| {
                    generic_param.intf_reqs.iter().for_each(|intf_req| {
                        s.expect_space(DefSpace::Type, |s| s.resolve_path(intf_req))
                            .emit_discard(&mut s.dctx)
                    })
                })
            });

            strct
                .fields
                .list
                .iter()
                .for_each(|field| s.resolve_ty(&field.ty).emit_discard(&mut s.dctx))
        });

        Ok(())
    }

    pub(crate) fn resolve_fn_sig(&mut self, hir_id: HirId, sig: &HirFnSig) -> ResolveResult {
        let scope_id = self.collector.scope_ctx.expect_hir_scope_id(hir_id);

        self.enter_scope(scope_id, |s| {
            sig.generic_params.as_ref().map(|generic_params| {
                generic_params.list.iter().for_each(|generic_param| {
                    generic_param.intf_reqs.iter().for_each(|intf_req| {
                        s.expect_space(DefSpace::Type, |s| s.resolve_path(intf_req))
                            .emit_discard(&mut s.dctx)
                    })
                })
            });

            sig.params
                .list
                .iter()
                .for_each(|param| s.resolve_ty(&param.ty).emit_discard(&mut s.dctx));

            sig.ret_ty
                .as_ref()
                .map(|ret_ty| s.resolve_ty(&ret_ty).emit_discard(&mut s.dctx));
        });

        Ok(())
    }

    pub(crate) fn resolve_fn(&mut self, item: &HirItem) -> ResolveResult {
        let func = item.expect_fn();
        self.resolve_fn_sig(item.id, &func.sig).emit(&mut self.dctx);

        let scope_id = self.collector.scope_ctx.expect_hir_scope_id(item.id);
        self.enter_scope(scope_id, |s| s.resolve_block(&func.body).emit(&mut s.dctx));

        Ok(())
    }

    pub(crate) fn resolve_var_sig(&mut self, hir_id: HirId, sig: &HirVarSig) -> ResolveResult {
        // For local level variables that are not normally collected
        // during collection
        self.collector
            .collect_var_sig(hir_id, sig, &mut self.dctx)
            .emit(&mut self.dctx);

        sig.ty
            .as_ref()
            .map(|ty| self.resolve_ty(&ty).emit_discard(&mut self.dctx));

        Ok(())
    }

    pub(crate) fn resolve_var_def(&mut self, item: &HirItem) -> ResolveResult {
        let def = item.expect_var_def();
        self.resolve_var_sig(item.id, &def.sig).emit(&mut self.dctx);

        def.val
            .as_ref()
            .map(|expr| self.resolve_expr(&expr).emit_discard(&mut self.dctx));

        Ok(())
    }

    pub(crate) fn resolve_block(&mut self, block: &HirBlock) -> ResolveResult {
        let scope_id = self.collector.scope_ctx.expect_hir_scope_id(block.id);
        self.enter_scope(scope_id, |s| {
            block
                .stmts
                .iter()
                .for_each(|stmt| s.resolve_stmt(&stmt).emit_discard(&mut s.dctx));
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

            res.map(|res| {
                resolution.replace(res);
                resolved += 1;
            });

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

    // std::vec::Vec<i32>::ElemeneType
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
        ident.arguments.as_ref().map(|arguments| {
            arguments.data.iter().for_each(|argument| {
                let res = match &argument {
                    HirIdentArgument::Expr(expr) => self.resolve_expr(&expr),
                    HirIdentArgument::Ty(ty) => self.resolve_ty(&ty),
                };

                res.emit(&mut self.dctx);
            })
        });

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
            self.dctx.error(
                ident.span,
                ResolverDiagErrorKind::Inaccessible(
                    name,
                    resolution.map(|_| SymbolKind::AssocItem),
                ),
            );
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
                tup.data
                    .iter()
                    .for_each(|el| s.resolve_ty(&el).emit_discard(&mut s.dctx));

                Ok(())
            }

            HirTyKind::Fn(func) => {
                func.params
                    .iter()
                    .for_each(|param| s.resolve_ty(&param).emit_discard(&mut s.dctx));

                func.ret_ty
                    .as_ref()
                    .map(|ret_ty| s.resolve_ty(&ret_ty).emit_discard(&mut s.dctx));

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

            HirExprKind::Cast(cast) => s.resolve_cast_expr(&cast),
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

    pub(crate) fn resolve_cast_expr(&mut self, cast: &HirCastExpr) -> ResolveResult {
        self.resolve_expr(&cast.expr).emit(&mut self.dctx);
        self.resolve_ty(&cast.ty)
    }

    pub(crate) fn resolve_assign_expr(
        &mut self,
        assignee: &HirExpr,
        expr: &HirExpr,
    ) -> ResolveResult {
        self.resolve_expr(&assignee).emit(&mut self.dctx);
        self.resolve_expr(&expr)
    }

    pub(crate) fn resolve_array_expr(&mut self, array: &HirArrayExpr) -> ResolveResult {
        array
            .elements
            .iter()
            .for_each(|el| self.resolve_expr(&el).emit_discard(&mut self.dctx));

        Ok(())
    }

    pub(crate) fn resolve_tuple_expr(&mut self, tup: &HirTupleExpr) -> ResolveResult {
        tup.elements
            .iter()
            .for_each(|el| self.resolve_expr(&el).emit_discard(&mut self.dctx));

        Ok(())
    }

    pub(crate) fn resolve_struct_expr(&mut self, strct: &HirStructExpr) -> ResolveResult {
        self.expect_space(DefSpace::Type, |s| {
            s.resolve_path(&strct.path).emit(&mut s.dctx)
        });

        strct.fields.iter().for_each(|field| {
            self.resolve_expr(&field.val).emit(&mut self.dctx);
        });

        Ok(())
    }

    pub(crate) fn resolve_anon_fn_expr(&mut self, expr: &HirExpr) -> ResolveResult {
        let HirExprKind::AnonFn(anfn) = &expr.kind else {
            unreachable!()
        };

        let scope_id = self.collector.scope_ctx.expect_hir_scope_id(expr.id);
        self.enter_scope(scope_id, |s| {
            anfn.params.list.iter().for_each(|param| {
                param.ty.map(|ty| s.resolve_ty(&ty).emit(&mut s.dctx));
            });

            anfn.ret_ty
                .map(|ret_ty| s.resolve_ty(&ret_ty).emit(&mut s.dctx));

            s.resolve_block(&anfn.body)
        })
    }

    pub(crate) fn resolve_fn_call_expr(&mut self, call: &HirFnCall) -> ResolveResult {
        self.resolve_expr(&call.callee).emit(&mut self.dctx);
        call.arguments
            .data
            .iter()
            .for_each(|argument| self.resolve_expr(&argument).emit_discard(&mut self.dctx));

        Ok(())
    }

    pub(crate) fn resolve_method_call_expr(&mut self, call: &HirMethodCall) -> ResolveResult {
        self.resolve_expr(&call.receiver).emit(&mut self.dctx);
        call.callee.arguments.as_ref().map(|arguments| {
            arguments.data.iter().for_each(|argument| {
                let res = match &argument {
                    HirIdentArgument::Expr(expr) => {
                        self.expect_space(DefSpace::Value, |s| s.resolve_expr(&expr))
                    }

                    HirIdentArgument::Ty(ty) => {
                        self.expect_space(DefSpace::Type, |s| s.resolve_ty(&ty))
                    }
                };

                res.emit(&mut self.dctx);
            })
        });

        call.arguments
            .data
            .iter()
            .for_each(|argument| self.resolve_expr(&argument).emit_discard(&mut self.dctx));

        Ok(())
    }

    pub(crate) fn resolve_if_expr(&mut self, ite: &HirIfExpr) -> ResolveResult {
        self.resolve_expr(&ite.cond).emit(&mut self.dctx);
        self.resolve_block(&ite.consequent).emit(&mut self.dctx);
        ite.alternate
            .as_ref()
            .map(|alt| self.resolve_block(&alt).emit(&mut self.dctx));

        Ok(())
    }
}
