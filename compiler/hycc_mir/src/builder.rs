use hycc_hir::{
    block::HirBlock,
    def::{AdtKind, DefId, DefKind, DefinitionTable},
    expr::{HirExpr, HirExprKind, HirFieldAccessFieldKind},
    item::{HirExtend, HirItem, HirItemKind, HirItemLevel, HirPetal},
    path::HirIdent,
    stmt::{HirPassStmt, HirRetStmt, HirStmt, HirStmtKind},
};
use hycc_span::Span;
use hycc_ty::{context::TyCtx, ty::TyKind};
use hycc_util::{bug, ternary};

use crate::{
    ctx::{MirDef, MirLoweringCtx},
    decl::{LocalDeclId, Mutability},
    scope::{MirScopeCtx, MirScopeId},
    stmt::{MirStatement, MirStatementKind, Operand, Place, Projection, RValue, RefKind},
    term::{MirTerminator, MirTerminatorKind},
};

#[derive(Debug)]
pub struct MirBuilder<'t, 'd> {
    pub ctx: MirLoweringCtx,
    pub scope_ctx: MirScopeCtx,

    tctx: &'t mut TyCtx,
    definitions: &'d DefinitionTable,
}

impl<'t, 'd> MirBuilder<'t, 'd> {
    pub fn new(tctx: &'t mut TyCtx, definitions: &'d DefinitionTable) -> Self {
        let mut inst = Self {
            ctx: MirLoweringCtx::new(),
            scope_ctx: MirScopeCtx::new(),
            tctx,
            definitions,
        };

        inst.build_ctx_mir_def_map();
        inst
    }

    fn build_ctx_mir_def_map(&mut self) {
        for (def_id, def) in self
            .definitions
            .defs()
            .iter()
            .enumerate()
            .map(|(i, def)| (DefId::new(i), def))
        {
            let mir_def = match &def.kind {
                DefKind::Var(var_def) if var_def.level == HirItemLevel::Top => {
                    let ty_id = self.tctx.expect_hir_ty_id(def.hir_id);
                    MirDef::Global(self.ctx.declare_global(ty_id, var_def.mutability, def.span))
                }

                DefKind::Fn(_) => MirDef::Body(def_id),

                _ => continue,
            };

            self.ctx.define(def_id, mir_def);
        }
    }

    fn read_place(&mut self, place: Place) -> Operand {
        // TODO
        Operand::Move(place)
    }

    pub fn lower(&mut self, tree: &HirItem) {
        if !matches!(&tree.kind, HirItemKind::Petal(_)) {
            bug!("invalid mir lowering! mir lowering must start at the tree (a petal)")
        };

        self.lower_item(&tree);
    }

    fn lower_item(&mut self, item: &HirItem) {
        match &item.kind {
            HirItemKind::Refer(_) => {}
            HirItemKind::Petal(petal) => self.lower_petal(&petal),
            HirItemKind::Proto(proto) => todo!("(mir) lower proto"),
            HirItemKind::Extend(extend) => self.lower_extend(&extend),
            HirItemKind::Struct(_) => {}
            HirItemKind::Fn(_) => self.lower_fn(&item),
            HirItemKind::VarDecl(_) => self.lower_var_decl(&item),
        }
    }

    fn lower_petal(&mut self, petal: &HirPetal) {
        for item in &petal.items {
            self.lower_item(&item)
        }
    }

    fn lower_extend(&mut self, extend: &HirExtend) {
        for item in &extend.items {
            self.lower_item(&item);
        }
    }

    fn lower_fn(&mut self, fn_item: &HirItem) {
        let HirItemKind::Fn(func) = &fn_item.kind else {
            unreachable!()
        };

        let Some(def_id) = self.definitions.get_def_id(fn_item.id) else {
            bug!("no def id found for hir id {:?}", fn_item.id)
        };

        let saved_scope_ctx = std::mem::take(&mut self.scope_ctx);
        let body_id = self.ctx.table.push_new_for(def_id);

        let unit_ty = self.tctx.make_unit_ty();
        let ret_ty = func.sig.ret_ty.map_or(unit_ty, |ret_ty| {
            self.tctx.get_hir_ty_id(ret_ty.id).unwrap_or(unit_ty)
        });

        let ret_local = self.ctx.table.get_mut(body_id).declare_local_ret(ret_ty);

        for param in &func.sig.params.list {
            let ty_id = self.tctx.expect_hir_ty_id(param.ty.id);
            let local_id = self.ctx.table.get_mut(body_id).declare_local_param(
                ty_id,
                Mutability::Mutable,
                param.span,
            );
            let param_def_id = self.definitions.get_def_id(param.id).unwrap();

            self.ctx.define(param_def_id, MirDef::Local(local_id));
        }

        self.lower_block(&func.body, &Place::local(ret_local));

        self.scope_ctx = saved_scope_ctx;
        self.ctx
            .table
            .get_mut(body_id)
            .try_attach_term(MirTerminator::new(MirTerminatorKind::Ret, Span::default()));
        self.ctx.table.pop();
    }

    fn lower_var_decl(&mut self, var_item: &HirItem) {
        let HirItemKind::VarDecl(decl) = &var_item.kind else {
            unreachable!()
        };

        let Some(expr) = decl.val else {
            return;
        };

        let ty_id = self.tctx.expect_hir_ty_id(var_item.id);
        let var_def_id = self.definitions.get_def_id(var_item.id).unwrap();

        if let Some(body) = self.ctx.table.top_mut() {
            let var_local_id = body.declare_local_var(ty_id, decl.mutability, decl.span);

            self.ctx.define(var_def_id, MirDef::Local(var_local_id));
            self.scope_ctx.top_mut().map(|top| top.store(var_local_id));

            let operand = self.lower_expr(&expr);
            let body = self.ctx.table.top_mut().unwrap();

            body.insert_stmt(MirStatement::new(
                MirStatementKind::StorageLive(var_local_id),
                var_item.span,
            ));

            body.insert_stmt(MirStatement::new(
                MirStatementKind::Assign(Box::new((
                    Place::local(var_local_id),
                    RValue::Use(operand),
                ))),
                var_item.span,
            ));
        } else {
            let body_id = self.ctx.table.push_new();
            let MirDef::Global(var_global_id) = self.ctx.expect_def(var_def_id) else {
                unreachable!()
            };

            let saved_scope_ctx = std::mem::take(&mut self.scope_ctx);
            let operand = self.lower_expr(&expr);

            self.ctx
                .table
                .get_mut(body_id)
                .insert_stmt(MirStatement::new(
                    MirStatementKind::Assign(Box::new((
                        Place::global(var_global_id),
                        RValue::Use(operand),
                    ))),
                    var_item.span,
                ));

            self.scope_ctx = saved_scope_ctx;
            self.ctx
                .table
                .get_mut(body_id)
                .try_attach_term(MirTerminator::new(MirTerminatorKind::Ret, Span::default()));
            self.ctx.table.pop();
        }
    }

    fn lower_block(&mut self, block: &HirBlock, dest: &Place) {
        let body_id = self.ctx.table.top_id().unwrap();

        let scope_id = self.scope_ctx.tree.create(self.scope_ctx.top_id());
        self.scope_ctx.push_id(scope_id);

        for stmt in &block.stmts {
            self.lower_stmt(&stmt, &dest);

            if !matches!(&stmt.kind, HirStmtKind::Ret(_) | HirStmtKind::Pass(_)) {
                continue;
            }

            break;
            // TODO: emit warning for unreachable statements or create a separate phase for it
        }

        self.scope_ctx
            .top_mut()
            .unwrap()
            .terminate(self.ctx.table.get_mut(body_id));

        self.scope_ctx.pop();
    }

    fn lower_stmt(&mut self, stmt: &HirStmt, dest: &Place) {
        match &stmt.kind {
            HirStmtKind::Ret(ret) => self.lower_ret_stmt(&ret),
            HirStmtKind::Pass(pass) => self.lower_pass_stmt(&pass, &dest),

            HirStmtKind::Item(item) => self.lower_item(&item),
            HirStmtKind::Expr(expr) => {
                // let ty = self.tctx.expect_hir_ty_id(expr.id);
                // let temp = Place::local(
                //     self.ctx
                //         .table
                //         .top_mut()
                //         .unwrap()
                //         .declare_local_temp(ty, expr.span),
                // );

                // self.lower_expr(&expr, Some(&temp));
                self.lower_expr(&expr);
            }
        }
    }

    fn lower_ret_stmt(&mut self, ret: &HirRetStmt) {
        let Some(body_id) = self.ctx.table.top_id() else {
            bug!("lowering statement without a currently existing definition!")
        };

        if let Some(val) = ret.value {
            let operand = self.lower_expr(&val);
            self.ctx
                .table
                .top_mut()
                .unwrap()
                .insert_stmt(MirStatement::new(
                    MirStatementKind::Assign(Box::new((
                        Place::local(LocalDeclId(0)),
                        RValue::Use(operand),
                    ))),
                    ret.span,
                ));
        }

        let body = self.ctx.table.get_mut(body_id);

        for (i, scope_id) in self
            .scope_ctx
            .stack()
            .iter()
            .cloned()
            .rev()
            .enumerate()
            .collect::<Vec<(usize, MirScopeId)>>()
        {
            let scope = self.scope_ctx.tree.get_mut(scope_id);
            // The top scope is always terminated as there will no longer be
            // any deeper scope from there, Scopes preceding the top will only
            // emit dead storage statements in the current scope.
            ternary!(i == 0, scope.terminate(body), scope.emit_dead(body));
        }

        body.attach_term(MirTerminator::new(MirTerminatorKind::Ret, ret.span));
        body.cue();
    }

    fn lower_pass_stmt(&mut self, pass: &HirPassStmt, dest: &Place) {
        let Some(pass_val) = pass.value else {
            return;
        };

        let operand = self.lower_expr(&pass_val);
        self.ctx
            .table
            .top_mut()
            .unwrap()
            .insert_stmt(MirStatement::new(
                MirStatementKind::Assign(Box::new((dest.clone(), RValue::Use(operand)))),
                pass.span,
            ));
    }

    fn lower_expr(&mut self, expr: &HirExpr) -> Operand {
        match &expr.kind {
            HirExprKind::Path(_) | HirExprKind::FieldAccess(_) => {
                let place = self.lower_place(&expr);
                self.read_place(place)
            }

            HirExprKind::Literal(lit) => Operand::Const(lit.const_id()),
            HirExprKind::RefExpr(_) => self.lower_ref_expr(&expr),

            HirExprKind::Binary(..) => self.lower_binary_expr(&expr),
            HirExprKind::Unary(_) => todo!("lower unary expr"),
            HirExprKind::Assign(..) => todo!("lower assign expr"),

            HirExprKind::Block(_) => self.lower_block_expr(&expr),

            HirExprKind::Array(_) => todo!("lower array expr"),
            HirExprKind::Tuple(_) => todo!("lower tuple expr"),
            HirExprKind::Struct(_) => self.lower_struct_expr(&expr),

            HirExprKind::AnonFn(_) => self.lower_anon_fn_expr(&expr),

            HirExprKind::FnCall(_) => self.lower_fn_call_expr(&expr),

            HirExprKind::MethodCall(_) => self.lower_method_call_expr(&expr),

            HirExprKind::If(_) => self.lower_if_expr(&expr),

            #[allow(unreachable_patterns)]
            _ => todo!(),
        }
    }

    fn lower_place(&mut self, expr: &HirExpr) -> Place {
        match &expr.kind {
            HirExprKind::Path(_) => self.lower_path(&expr),

            HirExprKind::FieldAccess(_) => self.lower_field_access_expr(&expr),

            // HirExprKind::FieldAccess { base, field } => {
            //     let mut place = self.lower_place_or_materialize(base);
            //     let field_idx = self.resolve_field_idx(base, field);
            //     place.projection.push(Projection::Field(field_idx));
            //     place
            // }
            _ => unreachable!(),
        }
    }

    fn lower_place_or_materialize(&mut self, expr: &HirExpr) -> Place {
        match &expr.kind {
            HirExprKind::Path(_) | HirExprKind::FieldAccess(_) => self.lower_place(&expr),
            _ => {
                let ty_id = self.tctx.expect_hir_ty_id(expr.id);
                let temp = Place::local(
                    self.ctx
                        .table
                        .top_mut()
                        .unwrap()
                        .declare_local_temp(ty_id, expr.span),
                );

                let operand = self.lower_expr(&expr);
                self.ctx
                    .table
                    .top_mut()
                    .unwrap()
                    .insert_stmt(MirStatement::new(
                        MirStatementKind::Assign(Box::new((temp.clone(), RValue::Use(operand)))),
                        expr.span,
                    ));

                temp
            }
        }
    }

    // TODO: attempt to define all def ids before usage
    fn lower_ident(&mut self, ident: &HirIdent) -> Place {
        // TODO: check ctx definitions
        match self
            .ctx
            .expect_def(self.definitions.get_def_id(ident.id).unwrap())
        {
            MirDef::Local(local_id) => Place::local(local_id),
            MirDef::Global(global_id) => Place::global(global_id),
            MirDef::Body(def_id) => {
                let body = self.ctx.table.top_mut().unwrap();
                let ty_id = self.tctx.expect_hir_ty_id(ident.id);

                let place = Place::local(body.declare_local_temp(ty_id, ident.span));
                self.ctx
                    .table
                    .top_mut()
                    .unwrap()
                    .insert_stmt(MirStatement::new(
                        MirStatementKind::Assign(Box::new((place.clone(), RValue::FnRef(def_id)))),
                        ident.span,
                    ));

                place
            }
        }
    }

    fn lower_path(&mut self, path_expr: &HirExpr) -> Place {
        let HirExprKind::Path(path) = &path_expr.kind else {
            unreachable!()
        };

        match self
            .ctx
            .expect_def(self.definitions.get_def_id(path.id).unwrap())
        {
            MirDef::Local(local_id) => Place::local(local_id),
            MirDef::Global(global_id) => Place::global(global_id),
            MirDef::Body(def_id) => {
                let body = self.ctx.table.top_mut().unwrap();
                let ty_id = self.tctx.expect_hir_ty_id(path_expr.id);

                let place = Place::local(body.declare_local_temp(ty_id, path.span));
                self.ctx
                    .table
                    .top_mut()
                    .unwrap()
                    .insert_stmt(MirStatement::new(
                        MirStatementKind::Assign(Box::new((place.clone(), RValue::FnRef(def_id)))),
                        path.span,
                    ));

                place
            }
        }
    }

    // fn lower_path_expr_rvalue(&mut self, path: &HirPath, dest: &Place) -> Option<RValue> {
    //     let rvalue = match self.lower_path(&path) {
    //         MirDef::Local(local_id) => RValue::Use(Operand::Move(Place::local(local_id))),
    //         MirDef::Global(global_id) => RValue::Use(Operand::Move(Place::global(global_id))),
    //         MirDef::Body(def_id) => RValue::FnRef(def_id),
    //     };

    //     self.ctx
    //         .table
    //         .top_mut()
    //         .unwrap()
    //         .insert_stmt(MirStatement::new(
    //             MirStatementKind::Assign(Box::new((dest.clone(), rvalue))),
    //             path.span,
    //         ));

    //     None
    // }

    fn lower_ref_expr(&mut self, ref_expr: &HirExpr) -> Operand {
        let HirExprKind::RefExpr(reference) = &ref_expr.kind else {
            unreachable!()
        };

        let kind = match &reference.mutability {
            Mutability::Mutable => RefKind::Mutable,
            Mutability::Immutable => RefKind::Immutable,
        };

        let ty_id = self.tctx.expect_hir_ty_id(ref_expr.id);
        let temp = Place::local(
            self.ctx
                .table
                .top_mut()
                .unwrap()
                .declare_local_temp(ty_id, ref_expr.span),
        );

        let place = self.lower_place_or_materialize(&reference.expr);
        self.ctx
            .table
            .top_mut()
            .unwrap()
            .insert_stmt(MirStatement::new(
                MirStatementKind::Assign(Box::new((temp.clone(), RValue::Ref(kind, place)))),
                ref_expr.span,
            ));

        self.read_place(temp)
    }

    fn lower_binary_expr(&mut self, binop_expr: &HirExpr) -> Operand {
        let HirExprKind::Binary(op, left, right) = &binop_expr.kind else {
            unreachable!()
        };

        let body = self.ctx.table.top_mut().unwrap();

        let (left_ty, right_ty) = (
            self.tctx.expect_hir_ty_id(left.id),
            self.tctx.expect_hir_ty_id(right.id),
        );
        let (left_temp, right_temp) = (
            Place::local(body.declare_local_temp(left_ty, left.span)),
            Place::local(body.declare_local_temp(right_ty, right.span)),
        );

        // TODO
        let left_operand = self.lower_expr(&left);
        self.ctx
            .table
            .top_mut()
            .unwrap()
            .insert_stmt(MirStatement::new(
                MirStatementKind::Assign(Box::new((left_temp.clone(), RValue::Use(left_operand)))),
                left.span,
            ));

        let right_operand = self.lower_expr(&right);
        self.ctx
            .table
            .top_mut()
            .unwrap()
            .insert_stmt(MirStatement::new(
                MirStatementKind::Assign(Box::new((
                    right_temp.clone(),
                    RValue::Use(right_operand),
                ))),
                right.span,
            ));

        let ty_id = self.tctx.expect_hir_ty_id(binop_expr.id);
        let temp = Place::local(
            self.ctx
                .table
                .top_mut()
                .unwrap()
                .declare_local_temp(ty_id, binop_expr.span),
        );

        self.ctx
            .table
            .top_mut()
            .unwrap()
            .insert_stmt(MirStatement::new(
                MirStatementKind::Assign(Box::new((
                    temp.clone(),
                    RValue::Binary(
                        *op,
                        Box::new((Operand::Move(left_temp), Operand::Move(right_temp))),
                    ),
                ))),
                binop_expr.span,
            ));

        self.read_place(temp)
    }

    fn lower_block_expr(&mut self, block_expr: &HirExpr) -> Operand {
        let HirExprKind::Block(block) = &block_expr.kind else {
            unreachable!()
        };

        let ty_id = self.tctx.expect_hir_ty_id(block_expr.id);
        let temp = Place::local(
            self.ctx
                .table
                .top_mut()
                .unwrap()
                .declare_local_temp(ty_id, block_expr.span),
        );

        self.lower_block(&block, &temp);

        self.read_place(temp)
    }

    fn lower_struct_expr(&mut self, struct_expr: &HirExpr) -> Operand {
        let HirExprKind::Struct(strct) = &struct_expr.kind else {
            unreachable!()
        };

        let body_id = self.ctx.table.top_id().unwrap();
        let operands = strct
            .fields
            .iter()
            .map(|field| self.lower_expr(&field.val))
            .collect::<Vec<_>>();

        let ty_id = self.tctx.expect_hir_ty_id(struct_expr.id);
        let temp = Place::local(
            self.ctx
                .table
                .get_mut(body_id)
                .declare_local_temp(ty_id, struct_expr.span),
        );

        self.ctx
            .table
            .get_mut(body_id)
            .insert_stmt(MirStatement::new(
                MirStatementKind::Assign(Box::new((
                    temp.clone(),
                    RValue::Aggregate(ty_id, operands),
                ))),
                struct_expr.span,
            ));

        self.read_place(temp)
    }

    fn lower_anon_fn_expr(&mut self, anfn_expr: &HirExpr) -> Operand {
        let HirExprKind::AnonFn(anfn) = &anfn_expr.kind else {
            unreachable!()
        };

        let saved_scope_ctx = std::mem::take(&mut self.scope_ctx);
        let body_id = self.ctx.table.push_new();

        let unit_ty = self.tctx.make_unit_ty();
        let ret_ty = anfn.ret_ty.map_or(unit_ty, |ret_ty| {
            self.tctx.get_hir_ty_id(ret_ty.id).unwrap_or(unit_ty)
        });

        let ret_local_id = self.ctx.table.get_mut(body_id).declare_local_ret(ret_ty);

        for param in &anfn.params.list {
            let Some(ty) = &param.ty else {
                continue;
            };

            let ty_id = self.tctx.expect_hir_ty_id(ty.id);
            let local_id = self.ctx.table.get_mut(body_id).declare_local_param(
                ty_id,
                Mutability::Mutable,
                param.span,
            );
            let param_def_id = self.definitions.get_def_id(param.id).unwrap();

            self.ctx.define(param_def_id, MirDef::Local(local_id));
        }

        self.lower_block(&anfn.body, &Place::local(ret_local_id));

        self.scope_ctx = saved_scope_ctx;
        self.ctx
            .table
            .get_mut(body_id)
            .try_attach_term(MirTerminator::new(MirTerminatorKind::Ret, Span::default()));
        self.ctx.table.pop();

        let body = self.ctx.table.top_mut().unwrap();

        let ty_id = self.tctx.expect_hir_ty_id(anfn_expr.id);
        let temp = Place::local(body.declare_local_temp(ty_id, anfn_expr.span));

        body.insert_stmt(MirStatement::new(
            MirStatementKind::Assign(Box::new((
                temp.clone(),
                RValue::AnonFn {
                    body_id,
                    captures: Vec::new(),
                },
            ))),
            anfn_expr.span,
        ));

        self.read_place(temp)
    }

    fn lower_fn_call_expr(&mut self, call_expr: &HirExpr) -> Operand {
        let HirExprKind::FnCall(call) = &call_expr.kind else {
            unreachable!()
        };

        let body_id = self.ctx.table.top_id().unwrap();

        // Callee
        let callee_operand = self.lower_expr(&call.callee);

        // Arguments
        let args = call
            .arguments
            .data
            .iter()
            .map(|arg| self.lower_expr(&arg))
            .collect::<Vec<_>>();

        let body = self.ctx.table.get_mut(body_id);
        let pos = body.basic_blocks.len() - 1;
        let next_block_id = body.insert_new();

        let ty_id = self.tctx.expect_hir_ty_id(call_expr.id);
        let temp = Place::local(body.declare_local_temp(ty_id, call_expr.span));

        body.basic_blocks
            .get_mut(pos)
            .unwrap()
            .terminator
            .replace(MirTerminator::new(
                MirTerminatorKind::Call {
                    func: callee_operand,
                    args,
                    dest: temp.clone(),
                    target: Some(next_block_id),
                    unwind: None, // TODO
                },
                call_expr.span,
            ));

        self.read_place(temp)
    }

    fn lower_field_access_expr(&mut self, access_expr: &HirExpr) -> Place {
        let HirExprKind::FieldAccess(access) = &access_expr.kind else {
            unreachable!()
        };

        let mut lead_ty_id = self.tctx.expect_hir_ty_id(access.leading.id);
        let mut lead_place = self.lower_place_or_materialize(&access.leading);

        let projection = loop {
            match self.tctx.get(lead_ty_id) {
                TyKind::Tuple(tup) => {
                    let HirFieldAccessFieldKind::Index(idx) = access.field.kind else {
                        unreachable!()
                    };

                    break Projection::Field(idx, tup[idx]);
                }

                TyKind::Adt(def_id, _) => {
                    let HirFieldAccessFieldKind::Ident(field_sym) = &access.field.kind else {
                        unreachable!()
                    };

                    let def = self.definitions.get(*def_id);
                    let DefKind::Adt(adt_kind) = &def.kind else {
                        unreachable!()
                    };

                    break match adt_kind {
                        AdtKind::Struct(struct_def) => {
                            let idx = *struct_def.field_map.get(field_sym).unwrap();
                            let field_ty = self.tctx.expect_hir_ty_id(struct_def.fields[idx].ty);
                            Projection::Field(idx, field_ty)
                        }
                    };
                }

                TyKind::Ref(ty_id, _) => {
                    lead_ty_id = *ty_id;
                }

                _ => unreachable!(),
            }
        };

        lead_place.projection.push(projection);
        lead_place
    }

    fn lower_method_call_expr(&mut self, call_expr: &HirExpr) -> Operand {
        let HirExprKind::MethodCall(call) = &call_expr.kind else {
            unreachable!()
        };

        let body_id = self.ctx.table.top_id().unwrap();

        // Arguments
        let args = std::iter::once(&call.receiver)
            .chain(call.arguments.data.iter())
            .map(|arg| self.lower_expr(&arg))
            .collect::<Vec<_>>();

        // Callee
        let place = self.lower_ident(&call.callee);
        let operand = self.read_place(place);
        let body = self.ctx.table.get_mut(body_id);
        let pos = body.basic_blocks.len() - 1;
        let next_block_id = body.insert_new();
        let ty_id = self.tctx.expect_hir_ty_id(call_expr.id);
        let temp = Place::local(body.declare_local_temp(ty_id, call_expr.span));

        body.basic_blocks
            .get_mut(pos)
            .unwrap()
            .terminator
            .replace(MirTerminator::new(
                MirTerminatorKind::Call {
                    func: operand,
                    args,
                    dest: temp.clone(),
                    target: Some(next_block_id),
                    unwind: None, // TODO
                },
                call_expr.span,
            ));

        self.read_place(temp)
    }

    fn lower_if_expr(&mut self, if_expr: &HirExpr) -> Operand {
        let HirExprKind::If(ite) = &if_expr.kind else {
            unreachable!()
        };

        let body_id = self.ctx.table.top_id().unwrap();
        let ty_id = self.tctx.expect_hir_ty_id(if_expr.id);
        let temp = Place::local(
            self.ctx
                .table
                .get_mut(body_id)
                .declare_local_temp(ty_id, if_expr.span),
        );

        let cond_ty_id = self.tctx.expect_hir_ty_id(ite.cond.id);
        let cond_place = Place::local(
            self.ctx
                .table
                .top_mut()
                .unwrap()
                .declare_local_temp(cond_ty_id, ite.cond.span),
        );

        let cond_operand = self.lower_expr(&ite.cond);
        self.ctx
            .table
            .top_mut()
            .unwrap()
            .insert_stmt(MirStatement::new(
                MirStatementKind::Assign(Box::new((cond_place.clone(), RValue::Use(cond_operand)))),
                ite.cond.span,
            ));

        let body = self.ctx.table.get_mut(body_id);
        let cond_bb_id = body.current_bb();

        // Consequent
        let conseq_start_bb_id = body.insert_new();
        self.lower_block(&ite.consequent, &temp);
        let conseq_end_bb_id = self.ctx.table.get(body_id).current_bb();

        // Alternate
        let alt_bb_id = ite.alternate.map(|alt| {
            let alt_start_bb_id = self.ctx.table.get_mut(body_id).insert_new();
            self.lower_block(&alt, &temp);

            (alt_start_bb_id, self.ctx.table.get(body_id).current_bb())
        });

        let body = self.ctx.table.get_mut(body_id);
        let join_bb_id = body.insert_new();

        body.get_mut(cond_bb_id)
            .terminator
            .replace(MirTerminator::new(
                MirTerminatorKind::SwitchInt {
                    discr: Operand::Move(cond_place.clone()),
                    targets: vec![
                        alt_bb_id
                            .map(|(alt_start_bb_id, _)| alt_start_bb_id)
                            .unwrap_or(join_bb_id),
                        conseq_start_bb_id,
                    ],
                },
                Span::default(),
            ));

        // Default branch joining for the consequent branch's ending basic block
        body.get_mut(conseq_end_bb_id)
            .terminator
            .get_or_insert(MirTerminator::new(
                MirTerminatorKind::Goto(join_bb_id),
                Span::default(),
            ));

        // Default branch joining for alternate branch's ending basic block
        if let Some((_, alt_end_bb_id)) = alt_bb_id {
            body.get_mut(alt_end_bb_id)
                .terminator
                .get_or_insert(MirTerminator::new(
                    MirTerminatorKind::Goto(join_bb_id),
                    Span::default(),
                ));
        }

        self.read_place(temp)
    }
}
