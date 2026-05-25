use std::collections::HashMap;

use hycc_hir::{
    block::HirBlock,
    def::{DefId, DefinitionTable},
    expr::{HirExpr, HirExprKind},
    item::{HirItem, HirItemKind, HirPetal},
    stmt::{HirStmt, HirStmtKind},
};
use hycc_span::Span;
use hycc_ty::context::TyCtx;
use hycc_util::bug;

use crate::{
    basic_block::{MirBasicBlock, MirBasicBlockId},
    body::MirBodyId,
    local::{LocalDeclId, Mutability},
    scope::{MirScopeId, MirScopeTerminator, MirScopeTree},
    stmt::{Location, MirStatement, MirStatementKind, Operand, RValue},
    table::MirTable,
    term::{MirTerminator, MirTerminatorKind},
};

#[derive(Debug)]
pub struct MirBuilder<'t, 'd> {
    pub table: MirTable,

    tctx: &'t mut TyCtx,
    definitions: &'d DefinitionTable,

    current_body: Option<MirBodyId>,
    def_map: HashMap<DefId, LocalDeclId>,

    pub scope_tree: MirScopeTree,
    current_scope: Option<MirScopeId>,

    // The local decl id to assign the value that the current block (scope) will yield
    block_assign_local: Option<LocalDeclId>,
}

impl<'t, 'd> MirBuilder<'t, 'd> {
    pub fn new(tctx: &'t mut TyCtx, definitions: &'d DefinitionTable) -> Self {
        Self {
            table: MirTable::new(),
            tctx,
            definitions,

            current_body: None,
            def_map: HashMap::new(),

            scope_tree: MirScopeTree::new(),
            current_scope: None,

            block_assign_local: None,
        }
    }

    fn emit_storage_init(&mut self, span: Span, local_id: LocalDeclId, rval: RValue) {
        let Some(body_id) = self.current_body else {
            bug!("storage initialization outside of a body!")
        };

        let body = self.table.get_body_mut(body_id);
        body.insert_stmt(MirStatement::new(
            MirStatementKind::StorageLive(local_id),
            span,
        ));

        let loc = Location::new(local_id);
        body.insert_stmt(MirStatement::new(
            MirStatementKind::Assign(Box::new((loc, rval))),
            span,
        ));

        let Some(current_scope) = self.current_scope else {
            return;
        };

        self.scope_tree
            .get_mut(current_scope)
            .store(local_id, MirBasicBlockId(body.basic_blocks.len() - 1));
    }

    pub fn lower(&mut self, tree: &HirPetal) {
        for item in &tree.items {
            self.lower_item(&item);
        }

        // dbg!(&self.scope_tree);
        // for def_id in self.table.defs().keys().map(|key| *key).collect::<Vec<_>>() {
        //     self.current_body.replace(def_id);
        //     self.emit_dead(None);
        //     self.current_body.take();
        // }
    }

    fn lower_item(&mut self, item: &HirItem) {
        match &item.kind {
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

        let Some(def_id) = self.definitions.get_def_id(fn_item.id).cloned() else {
            bug!("no def id found for hir id {:?}", fn_item.id)
        };

        let prev_body_id = self.current_body;

        let body_id = self.table.new_body_for(def_id);
        let body = self.table.get_body_mut(body_id);

        self.current_body = Some(body_id);

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

        body.declare_local_ret(ret_ty);

        for param in &func.params.list {
            let Some(ty) = self.tctx.get_ty_of_hir(param.ty.id) else {
                bug!("param {:?} has no attached ty!", param.id)
            };

            let local_id = body.declare_local_param(ty.id, Mutability::Mutable, param.span);
            let param_def_id = self.definitions.get_def_id(param.id).cloned().unwrap();

            self.def_map.insert(param_def_id, local_id);
        }

        self.lower_block(&func.body, Some(LocalDeclId(0)));

        self.table
            .get_body_mut(body_id)
            .attach_term(MirTerminator::new(MirTerminatorKind::Ret, Span::default()));

        self.current_body = prev_body_id;
    }

    fn lower_var_decl(&mut self, var_item: &HirItem) {
        let HirItemKind::VarDecl(decl) = &var_item.kind else {
            unreachable!()
        };

        let Some(body_id) = self.current_body else {
            return;
        };

        let body = self.table.get_body_mut(body_id);
        let Some(ty) = self.tctx.get_ty_of_hir(var_item.id) else {
            bug!("var decl {:?} does not have an attached ty!", var_item.id)
        };

        let var_local_id = body.declare_local_var(
            ty.id,
            Mutability::Immutable, // TODO: update to decl.mutability once implemented
            decl.span,
        );

        let var_val_local_id = decl.val.map(|val| self.lower_expr(&val));

        let var_def_id = *self.definitions.get_def_id(var_item.id).unwrap();
        self.def_map.insert(var_def_id, var_local_id);

        let Some(var_val_local_id) = var_val_local_id else {
            return;
        };

        self.emit_storage_init(
            var_item.span,
            var_local_id,
            RValue::Use(Operand::Move(Location::new(var_val_local_id))),
        );
    }

    fn lower_block(&mut self, block: &HirBlock, assign_local: Option<LocalDeclId>) {
        let prev_assign_local = self.block_assign_local;
        assign_local.map(|local_id| self.block_assign_local.replace(local_id));

        let prev_scope = self.current_scope;
        self.current_scope
            .replace(self.scope_tree.create(prev_scope));

        let scope_id = self.current_scope.unwrap();
        for stmt in &block.stmts {
            self.lower_stmt(&stmt);

            let scope_term = Some(match &stmt.kind {
                HirStmtKind::Ret(_) => MirScopeTerminator::Ret,
                HirStmtKind::Pass(_) => MirScopeTerminator::Normal,

                _ => continue,
            });

            self.scope_tree.get_mut(scope_id).term =
                scope_term.unwrap_or(MirScopeTerminator::Normal);

            break;
            // TODO: emit warning for unreachable statements or create a separate phase for it
        }

        let scope = self.scope_tree.get_mut(scope_id);
        let body = self.table.get_body_mut(self.current_body.unwrap());

        match scope.term {
            MirScopeTerminator::Normal => {
                for local_id in scope.local_decls().iter().rev() {
                    body.insert_stmt(MirStatement::new(
                        MirStatementKind::StorageDead(*local_id),
                        Span::default(),
                    ));
                }
            }

            MirScopeTerminator::Ret => {
                let mut curr_scope = Some(scope_id);
                while let Some(curr) = curr_scope {
                    let parent = self.scope_tree.get(curr);

                    for local_id in parent.local_decls().iter().rev() {
                        body.insert_stmt(MirStatement::new(
                            MirStatementKind::StorageDead(*local_id),
                            Span::default(),
                        ));
                    }

                    curr_scope = parent.parent;
                }

                body.cue();
            }
        }

        self.current_scope = prev_scope;
        self.block_assign_local = prev_assign_local;
    }

    fn lower_stmt(&mut self, stmt: &HirStmt) {
        let Some(body_id) = self.current_body else {
            bug!("lowering statement without a currently existing definition!")
        };

        match &stmt.kind {
            HirStmtKind::Ret(ret) => {
                let ret_val = ret
                    .value
                    .map(|val| Some(self.lower_expr(&val)))
                    .unwrap_or(None);

                let body = self.table.get_body_mut(body_id);

                // Emit assignment for the return LocalDecl (0) with
                // the return value.
                if let Some(local_id) = ret_val {
                    body.insert_stmt(MirStatement::new(
                        MirStatementKind::Assign(Box::new((
                            Location::new(LocalDeclId(0)),
                            RValue::Use(Operand::Move(Location::new(local_id))),
                        ))),
                        ret.span,
                    ));
                }

                body.attach_term(MirTerminator::new(MirTerminatorKind::Ret, ret.span));
            }

            HirStmtKind::Pass(pass) => {
                let pass_val = pass
                    .value
                    .map(|val| Some(self.lower_expr(&val)))
                    .unwrap_or(None);

                let body = self.table.get_body_mut(body_id);
                let assign_local = self.block_assign_local.unwrap();

                let Some(local_id) = pass_val else {
                    return;
                };

                body.insert_stmt(MirStatement::new(
                    MirStatementKind::Assign(Box::new((
                        Location::new(assign_local),
                        RValue::Use(Operand::Move(Location::new(local_id))),
                    ))),
                    pass.span,
                ));
            }

            HirStmtKind::Item(item) => self.lower_item(&item),
            HirStmtKind::Expr(expr) => {
                self.lower_expr(&expr);
            }
        }
    }

    fn lower_expr(&mut self, expr: &HirExpr) -> LocalDeclId {
        let Some(body_id) = self.current_body else {
            unreachable!()
        };

        let ty_id = self.tctx.get_ty_of_hir(expr.id).unwrap().id;
        let rvalue = self.lower_expr_rvalue(&expr);

        let body = self.table.get_body_mut(body_id);
        let local_id = body.declare_local_temp(ty_id, expr.span);

        let Some(rvalue) = rvalue else {
            return local_id;
        };

        let loc = Location::new(local_id);

        body.insert_stmt(MirStatement::new(
            MirStatementKind::Assign(Box::new((loc, rvalue))),
            expr.span,
        ));

        local_id
    }

    fn lower_expr_rvalue(&mut self, expr: &HirExpr) -> Option<RValue> {
        let ty_id = self.tctx.get_ty_of_hir(expr.id).unwrap().id;

        Some(match &expr.kind {
            HirExprKind::Path(path) => {
                let def_id = self.definitions.get_def_id(path.id).unwrap();
                let local_id = self.def_map.get(def_id).cloned().unwrap();

                RValue::Use(Operand::Move(Location::new(local_id)))
            }

            HirExprKind::RefExpr(reference) => todo!("lower ref expr"),

            HirExprKind::Literal(lit) => RValue::Use(Operand::Const(lit.const_id())),

            HirExprKind::Binary(op, left, right) => {
                let (left_loc, right_loc) = (
                    Location::new(self.lower_expr(&left)),
                    Location::new(self.lower_expr(&right)),
                );

                RValue::Binary(
                    *op,
                    Box::new((Operand::Move(left_loc), Operand::Move(right_loc))),
                )
            }

            HirExprKind::Unary(unary) => todo!("lower unary expr"),
            HirExprKind::Assign(assignee, expr) => todo!("lower assign expr"),

            HirExprKind::Block(block) => {
                let body_id = self.current_body.unwrap();
                let local_id = self
                    .table
                    .get_body_mut(body_id)
                    .declare_local_temp(ty_id, block.span);

                self.lower_block(&block, Some(local_id));

                RValue::Use(Operand::Move(Location::new(local_id)))
            }

            HirExprKind::Array(array) => todo!("lower array expr"),
            HirExprKind::Tuple(tup) => todo!("lower tuple expr"),
            HirExprKind::Struct(strct) => todo!("lower struct expr"),

            HirExprKind::AnonFn(anfn) => {
                let prev_body_id = self.current_body;

                let body_id = self.table.new_body();
                let body = self.table.get_body_mut(body_id);

                self.current_body = Some(body_id);

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

                body.declare_local_ret(ret_ty);

                for param in &anfn.params.list {
                    let Some(ty) = &param.ty else {
                        continue;
                    };

                    let Some(ty) = self.tctx.get_ty_of_hir(ty.id) else {
                        bug!("param {:?} has no attached ty!", param.id)
                    };

                    let local_id = body.declare_local_param(ty.id, Mutability::Mutable, param.span);
                    let param_def_id = self.definitions.get_def_id(param.id).cloned().unwrap();

                    self.def_map.insert(param_def_id, local_id);
                }

                self.lower_block(&anfn.body, Some(LocalDeclId(0)));

                self.table
                    .get_body_mut(body_id)
                    .attach_term(MirTerminator::new(MirTerminatorKind::Ret, Span::default()));

                self.current_body = prev_body_id;

                RValue::AnonFn {
                    body_id,
                    captures: Vec::new(),
                }
            }

            HirExprKind::FnCall(call) => {
                let body_id = self.current_body.unwrap();
                let body = self.table.get_body_mut(body_id);

                // body.attach_term(MirTerminator::new(MirTerminatorKind::Call { func: (), args: (), dest: () }))

                body.cue();

                //
                todo!("lower fn call expr")
            }

            HirExprKind::FieldAccess(access) => todo!("lower field access expr"),
            HirExprKind::MethodCall(call) => todo!("lower method call expr"),

            HirExprKind::If(ite) => {
                let body_id = self.current_body.unwrap();
                let cond_local_id = self.lower_expr(&ite.cond);

                let body = self.table.get_body_mut(body_id);
                let local_id = body.declare_local_temp(ty_id, expr.span);

                let cond_bb_id = body.current_bb();

                // Consequent
                let cons_bb_id = body.insert(MirBasicBlock::new());
                self.lower_block(&ite.consequent, Some(local_id));
                let inner_cons_bb_id = self.table.get_body(body_id).current_bb();

                // Alternate
                let alt_bb_id = ite.alternate.map(|alt| {
                    let alt_bb_id = self
                        .table
                        .get_body_mut(body_id)
                        .insert(MirBasicBlock::new());
                    self.lower_block(&alt, Some(local_id));
                    (alt_bb_id, self.table.get_body(body_id).current_bb())
                });

                let body = self.table.get_body_mut(body_id);
                let join_bb_id = body.insert(MirBasicBlock::new());

                body.get_mut(cond_bb_id)
                    .terminator
                    .replace(MirTerminator::new(
                        MirTerminatorKind::SwitchInt {
                            discr: Operand::Move(Location::new(cond_local_id)),
                            targets: vec![
                                alt_bb_id
                                    .map(|(_, alt_bb_id)| alt_bb_id)
                                    .unwrap_or(join_bb_id),
                                cons_bb_id,
                            ],
                        },
                        Span::default(),
                    ));

                // Default branch joining for the consequent branch's
                // latest inner basic block
                body.get_mut(inner_cons_bb_id)
                    .terminator
                    .get_or_insert(MirTerminator::new(
                        MirTerminatorKind::Goto(join_bb_id),
                        Span::default(),
                    ));

                // Default branch joining for alternate branch's
                // latest inner basic block
                if let Some((_, inner_alt_bb_id)) = alt_bb_id {
                    body.get_mut(inner_alt_bb_id)
                        .terminator
                        .get_or_insert(MirTerminator::new(
                            MirTerminatorKind::Goto(join_bb_id),
                            Span::default(),
                        ));
                }

                RValue::Use(Operand::Move(Location::new(local_id)))
            }
        })
    }
}
