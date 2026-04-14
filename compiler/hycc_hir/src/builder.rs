use hycc_ast::{
    Block, Expr, ExprKind, Identifier, Item, ItemKind, Path, Stmt, StmtKind, Ty, TyKind,
    expr::Unary,
    item::{Fn, FnParamList, Petal, PetalKind, VarDecl},
    path::{IdentifierArgument, IdentifierArguments},
    token::{Token, TokenKind},
    ty::Array,
};
use hycc_source::SourceRegistry;
use hycc_symbol::{Symbol, SymbolInterner};
use hycc_util::{digit_value, ternary};

use crate::{
    HirNode, HirTable,
    block::HirBlock,
    expr::{BinaryOp, HirExpr, HirExprKind, HirLiteral, HirUnary, UnaryOp},
    item::{
        HirFn, HirFnParam, HirFnParamList, HirItem, HirItemKind, HirPetal, HirPetalKind, HirVarDecl,
    },
    path::{HirIdent, HirIdentArgument, HirIdentArguments, HirPath, HirRawIdent},
    stmt::{HirStmt, HirStmtKind},
    ty::{HirArray, HirTy, HirTyKind},
};

#[derive(Debug)]
pub struct HirBuilder<'i, 's, 't, 'h>
where
    'h: 't,
{
    interner: &'i mut SymbolInterner,
    registry: &'s SourceRegistry,
    hir_table: &'t HirTable<'h>,
}

impl<'i, 's, 't, 'h> HirBuilder<'i, 's, 't, 'h> {
    pub fn new(
        interner: &'i mut SymbolInterner,
        source: &'s SourceRegistry,
        hir_table: &'t HirTable<'h>,
    ) -> Self {
        Self {
            interner,
            registry: source,
            hir_table,
        }
    }

    pub fn intern_str(&mut self, s: &str) -> Symbol {
        self.interner.intern(s)
    }

    pub fn intern_tok_str(&mut self, token: &Token) -> Symbol {
        let source = self.registry.get(token.span.src_id);
        self.intern_str(token.view(&source.data))
    }

    pub fn lower(&mut self, tree: Petal) -> &'h HirPetal<'h> {
        if let HirNode::Item(item) = self.hir_table.add(HirNode::Item(HirItem::new(
            HirItemKind::Petal(Box::new(HirPetal {
                kind: HirPetalKind::Root,
                items: tree
                    .items
                    .iter()
                    .map(|item| self.lower_item(&item))
                    .collect(),
                span: tree.span,
            })),
            tree.span,
        ))) {
            if let HirItemKind::Petal(petal) = &item.kind {
                &petal
            } else {
                unreachable!()
            }
        } else {
            unreachable!()
        }
    }

    fn lower_item(&mut self, item: &Item) -> &'h HirItem<'h> {
        let kind = match &item.kind {
            ItemKind::Petal(petal) => HirItemKind::Petal(Box::new(self.lower_petal(petal))),
            ItemKind::Fn(func) => HirItemKind::Fn(Box::new(self.lower_fn(&func))),
            ItemKind::VarDecl(decl) => HirItemKind::VarDecl(Box::new(self.lower_var_decl(&decl))),
        };

        let mut hir_item = HirItem::new(kind, item.span);
        hir_item.accessibility = item.accessibility;

        if let HirNode::Item(item) = self.hir_table.add(HirNode::Item(hir_item)) {
            item
        } else {
            unreachable!()
        }
    }

    fn lower_petal(&mut self, petal: &Petal) -> HirPetal<'h> {
        let kind = match &petal.kind {
            PetalKind::Root => HirPetalKind::Root,
            PetalKind::File(path, _) => HirPetalKind::File(self.lower_path(path)),
            PetalKind::Inline(path) => HirPetalKind::Inline(self.lower_path(path)),
        };

        HirPetal {
            kind,
            span: petal.span,
            items: petal
                .items
                .iter()
                .map(|item| self.lower_item(&item))
                .collect(),
        }
    }

    fn lower_fn(&mut self, func: &Fn) -> HirFn<'h> {
        HirFn {
            ident: self.lower_raw_ident(&func.ident),
            params: self.lower_fn_params(&func.params),
            ret_ty: func.ret_ty.as_ref().map(|ret_ty| self.lower_ty(ret_ty)),
            body: self.lower_block(&func.body),
            span: func.span(),
        }
    }

    fn lower_fn_params(&mut self, params: &FnParamList) -> HirFnParamList<'h> {
        let mut data = Vec::new();

        for param in &params.list {
            if let HirNode::FnParam(param) =
                self.hir_table.add(HirNode::FnParam(HirFnParam::<'h>::new(
                    self.lower_raw_ident(&param.ident),
                    self.lower_ty(&param.ty),
                    param.span(),
                )))
            {
                data.push(param);
            } else {
                unreachable!();
            }
        }

        HirFnParamList {
            list: data,
            span: params.span,
        }
    }

    pub fn lower_var_decl(&mut self, decl: &VarDecl) -> HirVarDecl<'h> {
        HirVarDecl {
            ident: self.lower_raw_ident(&decl.ident),
            ty: decl.ty.as_ref().map(|ty| self.lower_ty(ty)),
            val: decl.val.as_ref().map(|val| self.lower_expr(val)),
            span: decl.span(),
        }
    }

    fn lower_block(&mut self, block: &Block) -> &'h HirBlock<'h> {
        if let HirNode::Block(block) = self.hir_table.add(HirNode::Block(HirBlock::new(
            block
                .stmts
                .iter()
                .map(|stmt| self.lower_stmt(stmt))
                .collect(),
            block.span,
        ))) {
            block
        } else {
            unreachable!()
        }
    }

    fn lower_stmt(&mut self, stmt: &Stmt) -> &'h HirStmt<'h> {
        let kind = match &stmt.kind {
            StmtKind::Expr(expr) => HirStmtKind::Expr(self.lower_expr(expr)),
            StmtKind::Item(item) => HirStmtKind::Item(self.lower_item(item)),
        };

        if let HirNode::Stmt(stmt) = self
            .hir_table
            .add(HirNode::Stmt(HirStmt::new(kind, stmt.span)))
        {
            stmt
        } else {
            unreachable!()
        }
    }

    fn lower_expr(&mut self, expr: &Expr) -> &'h HirExpr<'h> {
        let kind = match &expr.kind {
            ExprKind::Path(path) => HirExprKind::Path(self.lower_path(&path)),
            ExprKind::Literal(lit) => HirExprKind::Literal(Box::new(self.lower_literal(lit))),
            ExprKind::Binary(op, left, right) => {
                let (op, left, right) = self.lower_binary(op, left, right);
                HirExprKind::Binary(op, left, right)
            }

            ExprKind::Unary(unary) => HirExprKind::Unary(Box::new(self.lower_unary(unary))),
            ExprKind::Assign(assignee, expr) => {
                HirExprKind::Assign(self.lower_expr(assignee), self.lower_expr(expr))
            }

            #[allow(unreachable_patterns)]
            _ => todo!(),
        };

        if let HirNode::Expr(expr) = self
            .hir_table
            .add(HirNode::Expr(HirExpr::new(kind, expr.span, expr.eval)))
        {
            expr
        } else {
            unreachable!()
        }
    }

    fn lower_literal(&mut self, lit: &Token) -> HirLiteral {
        let source = self.registry.get(lit.span.src_id);
        let view = lit.view(&source.data);

        match &lit.kind {
            TokenKind::Int { base } | TokenKind::Float { base } => {
                let is_negative = view.as_bytes()[0] == b'-';
                let view = ternary!(*base != 10, &view[(2 + (is_negative as usize))..], view);
                let mut split = view.split(".");
                let (integral, fractional) = (split.next().unwrap(), split.next());

                let mut val = u64::from_str_radix(integral, *base as u32).unwrap() as f64;

                if let Some(frac) = fractional {
                    let mut divisor = *base as u64;
                    val += frac
                        .as_bytes()
                        .iter()
                        .map(|byte| {
                            (digit_value(*byte, *base as u32) as f64)
                                * (1_f64 / (divisor as f64, divisor *= *base as u64).0)
                        })
                        .sum::<f64>();

                    HirLiteral::Float(val)
                } else {
                    HirLiteral::Int {
                        data: val as u64,
                        is_negative,
                    }
                }
            }

            TokenKind::Bool => HirLiteral::Bool(view == "true"),
            TokenKind::Char { .. } => HirLiteral::Char(view.as_bytes()[0]), // TODO: maybe
            // add support to
            // larger sized
            // characters
            TokenKind::String { .. } => HirLiteral::String(String::from(view)),

            _ => unreachable!(),
        }
    }

    fn lower_binary(
        &mut self,
        op: &Token,
        left: &Box<Expr>,
        right: &Box<Expr>,
    ) -> (BinaryOp, &'h HirExpr<'h>, &'h HirExpr<'h>) {
        (
            match &op.kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Percent => BinaryOp::Mod,
                TokenKind::CaretCaret => BinaryOp::Exp,

                TokenKind::EqEq => BinaryOp::Eq,
                TokenKind::BangEq => BinaryOp::Neq,
                TokenKind::Less => BinaryOp::Less,
                TokenKind::LessEq => BinaryOp::LessEq,
                TokenKind::Greater => BinaryOp::Greater,
                TokenKind::GreaterEq => BinaryOp::GreaterEq,

                TokenKind::AmpersandAmpersand => BinaryOp::And,
                TokenKind::PipePipe => BinaryOp::Or,

                TokenKind::Ampersand => BinaryOp::BitwiseAnd,
                TokenKind::Pipe => BinaryOp::BitwiseOr,
                TokenKind::Caret => BinaryOp::BitwiseXor,
                TokenKind::LessLess => BinaryOp::BitwiseLShift,
                TokenKind::GreaterGreater => BinaryOp::BitwiseRShift,

                _ => BinaryOp::Nop,
            },
            self.lower_expr(left),
            self.lower_expr(right),
        )
    }

    fn lower_unary(&mut self, expr: &Unary) -> HirUnary<'h> {
        match expr {
            Unary::Pre(op, expr) => HirUnary::Pre(
                match op.kind {
                    TokenKind::Minus => UnaryOp::Negative,
                    TokenKind::Bang => UnaryOp::Not,
                    TokenKind::Tilde => UnaryOp::BitwiseNot,

                    TokenKind::PlusPlus => UnaryOp::Increment,
                    TokenKind::MinusMinus => UnaryOp::Decrement,

                    _ => UnaryOp::Nop,
                },
                self.lower_expr(&expr),
            ),
            Unary::Post(op, expr) => HirUnary::Post(
                match op.kind {
                    TokenKind::PlusPlus => UnaryOp::Increment,
                    TokenKind::MinusMinus => UnaryOp::Decrement,

                    _ => UnaryOp::Nop,
                },
                self.lower_expr(&expr),
            ),
        }
    }

    fn lower_ty(&mut self, ty: &Ty) -> &'h HirTy<'h> {
        let kind = match &ty.kind {
            TyKind::Path(path) => HirTyKind::Path(self.lower_path(&path)),
            TyKind::Array(arr) => HirTyKind::Array(Box::new(self.lower_array(&arr))),
            TyKind::Unit(span) => HirTyKind::Unit(*span),
        };

        if let HirNode::Ty(ty) = self.hir_table.add(HirNode::Ty(HirTy::new(kind, ty.span))) {
            ty
        } else {
            unreachable!()
        }
    }

    fn lower_array(&mut self, arr: &Array) -> HirArray<'h> {
        HirArray {
            ty: self.lower_ty(&arr.ty),
            size: self.lower_expr(&arr.size),
            span: arr.span,
        }
    }

    fn lower_path(&mut self, path: &Path) -> &'h HirPath<'h> {
        if let HirNode::Path(path) = self.hir_table.add(HirNode::Path(HirPath::new(
            path.segments
                .iter()
                .map(|segment| self.lower_ident(segment))
                .collect(),
            path.span,
        ))) {
            path
        } else {
            unreachable!()
        }
    }

    fn lower_ident(&mut self, ident: &Identifier) -> &'h HirIdent<'h> {
        let ident = HirNode::Ident(HirIdent::new(
            self.lower_raw_ident(&ident.ident),
            ident
                .arguments
                .as_ref()
                .map(|args| self.lower_ident_args(&args)),
            ident.span,
        ));

        if let HirNode::Ident(ident) = self.hir_table.add(ident) {
            ident
        } else {
            unreachable!()
        }
    }

    fn lower_ident_args(&mut self, arguments: &IdentifierArguments) -> HirIdentArguments<'h> {
        let mut data = Vec::new();
        for argument in &arguments.data {
            data.push(match argument {
                IdentifierArgument::Expr(expr) => HirIdentArgument::Expr(self.lower_expr(&expr)),
                IdentifierArgument::Ty(ty) => HirIdentArgument::Ty(self.lower_ty(&ty)),
            })
        }

        HirIdentArguments {
            data,
            span: arguments.span,
        }
    }

    fn lower_raw_ident(&mut self, token: &Token) -> &'h HirRawIdent {
        let ident = HirNode::RawIdent(HirRawIdent::new(self.intern_tok_str(token), token.span));

        if let HirNode::RawIdent(raw) = self.hir_table.add(ident) {
            raw
        } else {
            unreachable!()
        }
    }
}
