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
    body::MirBody,
    local::{LocalDeclId, Mutability},
    stmt::{Location, MirStatement, MirStatementKind, Operand, RValue},
    table::MirTable,
    term::{MirTerminator, MirTerminatorKind},
};

#[derive(Debug)]
pub struct MirBuilder<'t, 'd> {
    pub table: MirTable,
    tctx: &'t mut TyCtx,
    definitions: &'d DefinitionTable,

    current_def: Option<DefId>,
    def_map: HashMap<DefId, LocalDeclId>,
}

impl<'t, 'd> MirBuilder<'t, 'd> {
    pub fn new(tctx: &'t mut TyCtx, definitions: &'d DefinitionTable) -> Self {
        Self {
            table: MirTable::new(),
            tctx,
            definitions,

            current_def: None,
            def_map: HashMap::new(),
        }
    }

    pub fn lower(&mut self, tree: &HirPetal) {
        for item in &tree.items {
            self.lower_item(&item);
        }
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

        let prev_def = self.current_def;

        let body = self.table.insert(def_id, MirBody::new());
        self.current_def = Some(def_id);

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

        self.lower_block(&func.body);

        let body = self.table.get_mut(def_id);
        if let Some(last) = body.basic_blocks.last_mut() {
            let ret = MirTerminator::new(MirTerminatorKind::Return, Span::default());
            let Some(term) = &last.terminator else {
                last.terminator.replace(ret);
                return;
            };

            let MirTerminatorKind::Pass(local_id) = &term.kind else {
                return;
            };

            let Some(local_id) = local_id else { return };

            last.statements.push(MirStatement::new(
                MirStatementKind::Assign(Box::new((
                    Location::new(LocalDeclId(0)),
                    RValue::Use(Operand::Move(Location::new(*local_id))),
                ))),
                Span::default(),
            ));

            last.terminator.replace(ret);
        }

        self.current_def = prev_def;
    }

    fn lower_var_decl(&mut self, var_item: &HirItem) {
        let HirItemKind::VarDecl(decl) = &var_item.kind else {
            unreachable!()
        };

        let Some(def_id) = self.current_def else {
            return;
        };

        let body = self.table.get_mut(def_id);
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

        let body = self.table.get_mut(def_id);
        let Some(last) = body.basic_blocks.last_mut() else {
            return;
        };

        last.statements.push(MirStatement::new(
            MirStatementKind::Assign(Box::new((
                Location::new(var_local_id),
                RValue::Use(Operand::Move(Location::new(var_val_local_id))),
            ))),
            var_item.span,
        ));
    }

    fn lower_block(&mut self, block: &HirBlock) -> MirBasicBlockId {
        let Some(def_id) = self.current_def else {
            unreachable!()
        };

        let body = self.table.get_mut(def_id);

        let block_id = body.insert(MirBasicBlock::new());
        if block_id.0 >= 1 {
            let Some(prev) = body.basic_blocks.get_mut(block_id.0 - 1) else {
                unreachable!()
            };

            prev.terminator.replace(MirTerminator::new(
                MirTerminatorKind::Goto(block_id),
                Span::default(),
            ));
        }

        for stmt in &block.stmts {
            self.lower_stmt(&stmt);
        }

        block_id
    }

    fn lower_stmt(&mut self, stmt: &HirStmt) {
        let Some(def_id) = self.current_def else {
            bug!("lowering statement without a currently existing definition!")
        };

        match &stmt.kind {
            HirStmtKind::Ret(_) => {
                //
                todo!("lower ret stmt")
            }

            HirStmtKind::Pass(pass) => {
                let pass_val = pass
                    .value
                    .map(|val| Some(self.lower_expr(&val)))
                    .unwrap_or(None);

                let body = self.table.get_mut(def_id);
                let Some(block) = body.basic_blocks.last_mut() else {
                    return;
                };

                block.terminator.replace(MirTerminator::new(
                    MirTerminatorKind::Pass(pass_val),
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
        let Some(def_id) = self.current_def else {
            unreachable!()
        };

        let ty_id = self.tctx.get_ty_of_hir(expr.id).unwrap().id;
        let rvalue = self.lower_expr_rvalue(&expr);

        let body = self.table.get_mut(def_id);
        let local_id = body.declare_local_temp(ty_id, expr.span);

        let Some(rvalue) = rvalue else {
            return local_id;
        };

        let Some(block) = body.basic_blocks.last_mut() else {
            bug!("lowering outside of a block!")
        };

        let loc = Location::new(local_id);

        block.statements.push(MirStatement::new(
            MirStatementKind::Assign(Box::new((loc, rvalue))),
            expr.span,
        ));

        local_id
    }

    fn lower_expr_rvalue(&mut self, expr: &HirExpr) -> Option<RValue> {
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
                let def_id = self.current_def?;
                let block_id = self.lower_block(&block);

                let body = self.table.get_mut(def_id);
                body.basic_blocks.push(MirBasicBlock::new());

                let block = body.get_mut(block_id);
                let Some(term) = &block.terminator else {
                    return None;
                };

                let MirTerminatorKind::Pass(local_id) = term.kind else {
                    return None;
                };

                let Some(local_id) = local_id else {
                    return None;
                };

                RValue::Use(Operand::Move(Location::new(local_id)))
            }

            HirExprKind::Array(array) => todo!("lower array expr"),
            HirExprKind::Tuple(tup) => todo!("lower tuple expr"),
            HirExprKind::Struct(strct) => todo!("lower struct expr"),
            HirExprKind::AnonFn(anfn) => todo!("lower anon fn expr"),
            HirExprKind::FnCall(call) => todo!("lower fn call expr"),
            HirExprKind::FieldAccess(access) => todo!("lower field access expr"),
            HirExprKind::MethodCall(call) => todo!("lower method call expr"),
        })
    }
}
