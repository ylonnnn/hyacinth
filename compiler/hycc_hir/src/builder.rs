use hycc_ast::{
    Block, Expr, ExprKind, Identifier, Item, ItemKind, Path, Program, Stmt, StmtKind, Ty, TyKind,
    expr::Unary,
    item::{Fn, FnParamList, VarDecl},
    path::{IdentifierArgument, IdentifierArguments},
    token::{Token, TokenKind},
    ty::Array,
};
use hycc_source::Source;
use hycc_symbol::{Symbol, SymbolInterner};

use crate::{
    HirId,
    block::HirBlock,
    expr::{BinaryOp, HirExpr, HirExprKind, HirLiteral, HirUnary, UnaryOp},
    item::{HirFn, HirFnParam, HirFnParamList, HirItem, HirItemKind, HirVarDecl},
    path::{HirIdent, HirIdentArgument, HirIdentArguments, HirPath, HirRawIdent},
    program::HirProgram,
    stmt::{HirStmt, HirStmtKind},
    ty::{HirArray, HirTy, HirTyKind},
};

#[derive(Debug)]
pub struct HirBuilder<'s> {
    interner: SymbolInterner,
    source: &'s Source,

    counter: usize,
}

impl<'s> HirBuilder<'s> {
    pub fn new(source: &'s Source) -> Self {
        Self {
            interner: SymbolInterner::new(),
            source,
            counter: 0,
        }
    }

    pub fn intern_tok_str(&mut self, token: &Token) -> Symbol {
        self.interner.intern(token.view(&self.source.data))
    }

    fn next_id(&mut self) -> HirId {
        HirId((self.counter, self.counter += 1).0)
    }

    pub fn lower(&mut self, tree: Program) -> HirProgram {
        let mut hir_tree = HirProgram {
            id: self.next_id(),
            items: Vec::new(),
        };

        for item in tree.items {
            hir_tree.items.push(self.lower_item(&item));
        }

        hir_tree
    }

    fn lower_item(&mut self, item: &Item) -> HirItem {
        let kind = match &item.kind {
            ItemKind::Fn(func) => HirItemKind::Fn(Box::new(self.lower_fn(&func))),
            ItemKind::VarDecl(decl) => HirItemKind::VarDecl(Box::new(self.lower_var_decl(&decl))),
        };

        HirItem {
            id: self.next_id(),
            kind,
            span: item.span,
        }
    }

    fn lower_fn(&mut self, func: &Fn) -> HirFn {
        HirFn {
            ident: self.lower_raw_ident(&func.ident),
            params: self.lower_fn_params(&func.params),
            ret_ty: func.ret_ty.as_ref().map(|ret_ty| self.lower_ty(ret_ty)),
            body: self.lower_block(&func.body),
            span: func.span(),
        }
    }

    fn lower_fn_params(&mut self, params: &FnParamList) -> HirFnParamList {
        let mut data = Vec::new();

        for param in &params.list {
            data.push(HirFnParam {
                ident: self.lower_raw_ident(&param.ident),
                ty: Box::new(self.lower_ty(&param.ty)),
                span: param.span(),
            })
        }

        HirFnParamList {
            list: data,
            span: params.span,
        }
    }

    pub fn lower_var_decl(&mut self, decl: &VarDecl) -> HirVarDecl {
        HirVarDecl {
            id: self.next_id(),
            ident: self.lower_raw_ident(&decl.ident),
            ty: decl.ty.as_ref().map(|ty| Box::new(self.lower_ty(ty))),
            val: decl.val.as_ref().map(|val| Box::new(self.lower_expr(val))),
            span: decl.span(),
        }
    }

    fn lower_block(&mut self, block: &Block) -> HirBlock {
        HirBlock {
            id: self.next_id(),
            stmts: block
                .stmts
                .iter()
                .map(|stmt| self.lower_stmt(stmt))
                .collect(),
            span: block.span,
        }
    }

    fn lower_stmt(&mut self, stmt: &Stmt) -> HirStmt {
        let kind = match &stmt.kind {
            StmtKind::Expr(expr) => HirStmtKind::Expr(Box::new(self.lower_expr(expr))),
            StmtKind::Item(item) => HirStmtKind::Item(Box::new(self.lower_item(item))),
        };

        HirStmt {
            id: self.next_id(),
            kind,
            span: stmt.span,
        }
    }

    fn lower_expr(&mut self, expr: &Expr) -> HirExpr {
        let kind = match &expr.kind {
            ExprKind::Path(path) => HirExprKind::Path(Box::new(self.lower_path(&path))),
            ExprKind::Literal(lit) => HirExprKind::Literal(Box::new(self.lower_literal(lit))),
            ExprKind::Binary(op, left, right) => {
                let (op, left, right) = self.lower_binary(op, left, right);
                HirExprKind::Binary(op, left, right)
            }

            ExprKind::Unary(unary) => HirExprKind::Unary(Box::new(self.lower_unary(unary))),
            ExprKind::Assign(assignee, expr) => HirExprKind::Assign(
                Box::new(self.lower_expr(assignee)),
                Box::new(self.lower_expr(expr)),
            ),

            #[allow(unreachable_patterns)]
            _ => todo!(),
        };

        HirExpr {
            id: self.next_id(),
            kind,
            span: expr.span,
            eval: expr.eval,
        }
    }

    fn lower_literal(&mut self, lit: &Token) -> HirLiteral {
        let view = lit.view(&self.source.data);

        match &lit.kind {
            TokenKind::Int { base } | TokenKind::Float { base } => {
                todo!("{view} of base {base}")
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
    ) -> (BinaryOp, Box<HirExpr>, Box<HirExpr>) {
        (
            match &op.kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Add,
                TokenKind::Star => BinaryOp::Add,
                TokenKind::Slash => BinaryOp::Add,
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
                TokenKind::Pipe => BinaryOp::Or,
                TokenKind::Caret => BinaryOp::BitwiseXor,
                TokenKind::LessLess => BinaryOp::BitwiseLShift,
                TokenKind::GreaterGreater => BinaryOp::BitwiseRShift,

                _ => BinaryOp::Nop,
            },
            Box::new(self.lower_expr(left)),
            Box::new(self.lower_expr(right)),
        )
    }

    fn lower_unary(&mut self, expr: &Unary) -> HirUnary {
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
                Box::new(self.lower_expr(&expr)),
            ),
            Unary::Post(op, expr) => HirUnary::Post(
                match op.kind {
                    TokenKind::PlusPlus => UnaryOp::Increment,
                    TokenKind::MinusMinus => UnaryOp::Decrement,

                    _ => UnaryOp::Nop,
                },
                Box::new(self.lower_expr(&expr)),
            ),
        }
    }

    fn lower_ty(&mut self, ty: &Ty) -> HirTy {
        let kind = match &ty.kind {
            TyKind::Path(path) => HirTyKind::Path(Box::new(self.lower_path(&path))),
            TyKind::Array(arr) => HirTyKind::Array(Box::new(self.lower_array(&arr))),
            TyKind::Unit(span) => HirTyKind::Unit(*span),
        };

        HirTy {
            id: self.next_id(),
            kind,
            span: ty.span,
        }
    }

    fn lower_array(&mut self, arr: &Array) -> HirArray {
        HirArray {
            ty: Box::new(self.lower_ty(&arr.ty)),
            size: Box::new(self.lower_expr(&arr.size)),
            span: arr.span,
        }
    }

    fn lower_path(&mut self, path: &Path) -> HirPath {
        HirPath {
            id: self.next_id(),
            segments: path
                .segments
                .iter()
                .map(|segment| self.lower_ident(segment))
                .collect(),
            span: path.span,
        }
    }

    fn lower_ident(&mut self, ident: &Identifier) -> HirIdent {
        HirIdent {
            id: self.next_id(),
            ident: self.lower_raw_ident(&ident.ident),
            arguments: ident
                .arguments
                .as_ref()
                .map(|args| self.lower_ident_args(&args)),
            span: ident.span,
        }
    }

    fn lower_ident_args(&mut self, arguments: &IdentifierArguments) -> HirIdentArguments {
        let mut data = Vec::new();
        for argument in &arguments.data {
            data.push(match argument {
                IdentifierArgument::Expr(expr) => {
                    HirIdentArgument::Expr(Box::new(self.lower_expr(&expr)))
                }
                IdentifierArgument::Ty(ty) => HirIdentArgument::Ty(Box::new(self.lower_ty(&ty))),
            })
        }

        HirIdentArguments {
            data,
            span: arguments.span,
        }
    }

    fn lower_raw_ident(&mut self, token: &Token) -> HirRawIdent {
        HirRawIdent {
            id: self.next_id(),
            ident: self.intern_tok_str(token),
            span: token.span,
        }
    }
}
