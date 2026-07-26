use hycc_hir::{
    block::HirBlock,
    def::DefinitionTable,
    expr::{BinaryOp, HirAnonFn, HirExpr, HirExprKind},
    item::{HirItem, HirItemKind, HirPetal},
    path::HirPath,
    stmt::{HirPassStmt, HirRetStmt, HirStmt, HirStmtKind},
};
use hycc_span::Span;
use hycc_ty::context::TyCtx;
use hycc_util::{bug, ternary};

use crate::{
    ctx::{MirDef, MirLoweringCtx},
    decl::{LocalDeclId, Mutability},
    scope::{MirScopeCtx, MirScopeId},
    stmt::{MirStatement, MirStatementKind, Operand, Place, RValue, RefKind},
    term::{MirTerminator, MirTerminatorKind},
};

#[derive(Debug)]
pub struct MirBuilder<'t, 'd> {
    pub ctx: MirLoweringCtx,

    tctx: &'t mut TyCtx,
    definitions: &'d DefinitionTable,

    pub scope_ctx: MirScopeCtx,
}

impl<'t, 'd> MirBuilder<'t, 'd> {
    pub fn new(tctx: &'t mut TyCtx, definitions: &'d DefinitionTable) -> Self {
        Self {
            ctx: MirLoweringCtx::new(),

            tctx,
            definitions,

            scope_ctx: MirScopeCtx::new(),
        }
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

    fn lower_fn(&mut self, fn_item: &HirItem) {
        let HirItemKind::Fn(func) = &fn_item.kind else {
            unreachable!()
        };

        let Some(def_id) = self.definitions.get_def_id(fn_item.id) else {
            bug!("no def id found for hir id {:?}", fn_item.id)
        };

        let saved_scope_ctx = std::mem::take(&mut self.scope_ctx);
        let body_id = self.ctx.table.push_new_for(def_id);

        self.ctx.define(def_id, MirDef::Body(def_id));

        let unit_ty = self.tctx.make_unit_ty();
        let ret_ty = func
            .ret_ty
            .map(|ret_ty| {
                self.tctx
                    .get_ty_of_hir(ret_ty.id)
                    .map(|ty| ty.id)
                    .unwrap_or(unit_ty)
            })
            .unwrap_or(unit_ty);

        let ret_local = self.ctx.table.get_mut(body_id).declare_local_ret(ret_ty);

        for param in &func.params.list {
            let Some(ty) = self.tctx.get_ty_of_hir(param.ty.id) else {
                bug!("param {:?} has no attached ty!", param.id)
            };

            let local_id = self.ctx.table.get_mut(body_id).declare_local_param(
                ty.id,
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

        let ty = self.tctx.get_ty_of_hir(var_item.id).unwrap();
        let var_def_id = self.definitions.get_def_id(var_item.id).unwrap();

        if let Some(body) = self.ctx.table.top_mut() {
            let var_local_id = body.declare_local_var(ty.id, decl.mutability, decl.span);

            body.insert_stmt(MirStatement::new(
                MirStatementKind::StorageLive(var_local_id),
                var_item.span,
            ));

            self.ctx.define(var_def_id, MirDef::Local(var_local_id));
            self.scope_ctx.top_mut().map(|top| top.store(var_local_id));
            self.lower_expr(&expr, &Place::local(var_local_id));
        } else {
            let body_id = self.ctx.table.push_new();
            let var_global_id = self.ctx.declare_global(ty.id, decl.mutability, decl.span);
            self.ctx.define(var_def_id, MirDef::Global(var_global_id));
            let saved_scope_ctx = std::mem::take(&mut self.scope_ctx);

            self.lower_expr(&expr, &Place::global(var_global_id));

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
                self.lower_expr(&expr, &dest);
            }
        }
    }

    fn lower_ret_stmt(&mut self, ret: &HirRetStmt) {
        let Some(body_id) = self.ctx.table.top_id() else {
            bug!("lowering statement without a currently existing definition!")
        };

        let dest = Place::local(LocalDeclId(0));
        if let Some(val) = ret.value {
            self.lower_expr(&val, &dest);
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

        self.lower_expr(&pass_val, &dest);
    }

    fn lower_expr(&mut self, expr: &HirExpr, dest: &Place) {
        let Some(body_id) = self.ctx.table.top_id() else {
            unreachable!()
        };

        let Some(rvalue) = self.lower_expr_rvalue(&expr, &dest) else {
            return;
        };

        self.ctx
            .table
            .get_mut(body_id)
            .insert_stmt(MirStatement::new(
                MirStatementKind::Assign(Box::new((dest.clone(), rvalue))),
                expr.span,
            ));
    }

    fn lower_expr_rvalue(&mut self, expr: &HirExpr, dest: &Place) -> Option<RValue> {
        Some(match &expr.kind {
            HirExprKind::Path(path) => self.lower_path_expr_rvalue(&path, &dest)?,
            HirExprKind::RefExpr(_) => self.lower_ref_expr_rvalue(&expr),
            HirExprKind::Literal(lit) => {
                let body = self.ctx.table.top_mut()?;
                body.insert_stmt(MirStatement::new(
                    MirStatementKind::Assign(Box::new((
                        dest.clone(),
                        RValue::Use(Operand::Const(lit.const_id())),
                    ))),
                    expr.span,
                ));

                None
            }?,
            HirExprKind::Binary(op, left, right) => {
                self.lower_binary_expr_rvalue(*op, &left, &right)
            }

            HirExprKind::Unary(unary) => todo!("lower unary expr"),
            HirExprKind::Assign(assignee, expr) => todo!("lower assign expr"),

            HirExprKind::Block(_) => self.lower_block_expr_rvalue(&expr, &dest)?,

            HirExprKind::Array(array) => todo!("lower array expr"),
            HirExprKind::Tuple(tup) => todo!("lower tuple expr"),
            HirExprKind::Struct(strct) => todo!("lower struct expr"),

            HirExprKind::AnonFn(anfn) => self.lower_anon_fn_expr_rvalue(&anfn),

            HirExprKind::FnCall(_) => self.lower_fn_call_expr_rvalue(&expr, &dest)?,

            HirExprKind::FieldAccess(access) => todo!("lower field access expr"),
            HirExprKind::MethodCall(call) => todo!("lower method call expr"),

            HirExprKind::If(_) => self.lower_if_expr_rvalue(&expr, &dest)?,
        })
    }

    fn lower_path(&mut self, path: &HirPath) -> MirDef {
        self.ctx
            .get_def(self.definitions.get_def_id(path.id).unwrap())
    }

    fn lower_path_expr_rvalue(&mut self, path: &HirPath, dest: &Place) -> Option<RValue> {
        // TODO: improve?
        let rvalue = match self.lower_path(&path) {
            MirDef::Local(local_id) => RValue::Use(Operand::Move(Place::local(local_id))),
            MirDef::Global(global_id) => RValue::Use(Operand::Move(Place::global(global_id))),
            MirDef::Body(def_id) => RValue::FnRef(def_id),
        };

        self.ctx
            .table
            .top_mut()
            .unwrap()
            .insert_stmt(MirStatement::new(
                MirStatementKind::Assign(Box::new((dest.clone(), rvalue))),
                path.span,
            ));

        None
    }

    fn lower_ref_expr_rvalue(&mut self, ref_expr: &HirExpr) -> RValue {
        let HirExprKind::RefExpr(reference) = &ref_expr.kind else {
            unreachable!()
        };

        let kind = match &reference.mutability {
            Mutability::Mutable => RefKind::Mutable,
            Mutability::Immutable => RefKind::Immutable,
        };

        match &reference.expr.kind {
            HirExprKind::Path(path) => match self.lower_path(&path) {
                MirDef::Local(local_id) => RValue::Ref(kind, Place::local(local_id)),
                MirDef::Global(global_id) => RValue::Ref(kind, Place::global(global_id)),
                MirDef::Body(def_id) => RValue::FnRef(def_id),
            },

            HirExprKind::RefExpr(_) => self.lower_ref_expr_rvalue(&reference.expr),

            _ => {
                let ty_id = self.tctx.get_ty_of_hir(ref_expr.id).unwrap().id;
                let expr_place = Place::local(
                    self.ctx
                        .table
                        .top_mut()
                        .unwrap()
                        .declare_local_temp(ty_id, ref_expr.span),
                );

                self.lower_expr(&reference.expr, &expr_place);

                RValue::Ref(kind, expr_place.clone())
            }
        }
    }

    fn lower_binary_expr_rvalue(
        &mut self,
        op: BinaryOp,
        left: &HirExpr,
        right: &HirExpr,
    ) -> RValue {
        let body = self.ctx.table.top_mut().unwrap();

        let (left_ty, right_ty) = (
            self.tctx.get_ty_of_hir(left.id).unwrap().id,
            self.tctx.get_ty_of_hir(right.id).unwrap().id,
        );
        let (left_dest, right_dest) = (
            Place::local(body.declare_local_temp(left_ty, left.span)),
            Place::local(body.declare_local_temp(right_ty, right.span)),
        );

        self.lower_expr(&left, &left_dest);
        self.lower_expr(&right, &right_dest);

        RValue::Binary(
            op,
            Box::new((Operand::Move(left_dest), Operand::Move(right_dest))),
        )
    }

    fn lower_block_expr_rvalue(&mut self, block_expr: &HirExpr, dest: &Place) -> Option<RValue> {
        let HirExprKind::Block(block) = &block_expr.kind else {
            unreachable!()
        };

        self.lower_block(&block, &dest);
        None
    }

    fn lower_anon_fn_expr_rvalue(&mut self, anfn: &HirAnonFn) -> RValue {
        let saved_scope_ctx = std::mem::take(&mut self.scope_ctx);
        let body_id = self.ctx.table.push_new();

        let unit_ty = self.tctx.make_unit_ty();
        let ret_ty = anfn
            .ret_ty
            .map(|ret_ty| {
                self.tctx
                    .get_ty_of_hir(ret_ty.id)
                    .map(|ty| ty.id)
                    .unwrap_or(unit_ty)
            })
            .unwrap_or(unit_ty);

        let ret_local_id = self.ctx.table.get_mut(body_id).declare_local_ret(ret_ty);

        for param in &anfn.params.list {
            let Some(ty) = &param.ty else {
                continue;
            };

            let Some(ty) = self.tctx.get_ty_of_hir(ty.id) else {
                bug!("param {:?} has no attached ty!", param.id)
            };

            let local_id = self.ctx.table.get_mut(body_id).declare_local_param(
                ty.id,
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

        RValue::AnonFn {
            body_id,
            captures: Vec::new(),
        }
    }

    fn lower_fn_call_expr_rvalue(&mut self, call_expr: &HirExpr, dest: &Place) -> Option<RValue> {
        let HirExprKind::FnCall(call) = &call_expr.kind else {
            unreachable!()
        };

        let body_id = self.ctx.table.top_id().unwrap();
        let body = self.ctx.table.get_mut(body_id);
        let ty_id = self.tctx.get_ty_of_hir(call_expr.id).unwrap().id;

        // Callee
        let callee_place = Place::local(body.declare_local_temp(ty_id, call.callee.span));
        self.lower_expr(&call.callee, &callee_place);

        // Arguments
        let args = call
            .arguments
            .data
            .iter()
            .map(|arg| {
                let Some(ty) = self.tctx.get_ty_of_hir(arg.id).map(|ty| ty.id) else {
                    bug!("argument {:?} is expected to have an attached type", arg.id);
                };

                let arg_dest = Place::local(
                    self.ctx
                        .table
                        .get_mut(body_id)
                        .declare_local_temp(ty, arg.span),
                );

                self.lower_expr(&arg, &arg_dest);
                Operand::Move(arg_dest)
            })
            .collect::<Vec<_>>();

        let body = self.ctx.table.get_mut(body_id);
        let pos = body.basic_blocks.len() - 1;
        let next_block_id = body.insert_new();

        body.basic_blocks
            .get_mut(pos)
            .unwrap()
            .terminator
            .replace(MirTerminator::new(
                MirTerminatorKind::Call {
                    func: Operand::Move(callee_place),
                    args,
                    dest: dest.clone(),
                    target: Some(next_block_id),
                    unwind: None, // TODO
                },
                call_expr.span,
            ));

        None
    }

    fn lower_if_expr_rvalue(&mut self, if_expr: &HirExpr, dest: &Place) -> Option<RValue> {
        let HirExprKind::If(ite) = &if_expr.kind else {
            unreachable!()
        };

        let body_id = self.ctx.table.top_id().unwrap();
        let ty_id = self.tctx.get_ty_of_hir(if_expr.id).unwrap().id;

        let cond_place = Place::local(
            self.ctx
                .table
                .top_mut()
                .unwrap()
                .declare_local_temp(ty_id, ite.cond.span),
        );
        self.lower_expr(&ite.cond, &cond_place);

        let body = self.ctx.table.get_mut(body_id);
        let cond_bb_id = body.current_bb();

        // Consequent
        let conseq_start_bb_id = body.insert_new();
        self.lower_block(&ite.consequent, &dest);
        let conseq_end_bb_id = self.ctx.table.get(body_id).current_bb();

        // Alternate
        let alt_bb_id = ite.alternate.map(|alt| {
            let alt_start_bb_id = self.ctx.table.get_mut(body_id).insert_new();
            self.lower_block(&alt, &dest);

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

        None
    }
}
