use hycc_ast::{
    Expr, ExprKind, Identifier, Item, ItemKind, Path, Program, Ty, TyKind,
    expr::Unary,
    item::{Fn, FnParamList},
    path::{IdentifierArgument, IdentifierArguments},
    token::{Token, TokenKind},
    ty::Array,
};
use hycc_source::Source;
use hycc_symbol::{Symbol, SymbolInterner};

use crate::{
    expr::{BinaryOp, HirExpr, HirExprKind, HirLiteral, HirUnary, UnaryOp},
    item::{HirFn, HirFnParam, HirFnParamList, HirItem, HirItemKind},
    path::{HirIdent, HirIdentArgument, HirIdentArguments, HirPath, HirRawIdent},
    program::HirProgram,
    ty::{HirArray, HirTy, HirTyKind},
};

#[derive(Debug)]
pub struct HirBuilder<'s> {
    interner: SymbolInterner,
    source: &'s Source,
}

impl<'s> HirBuilder<'s> {
    pub fn new(source: &'s Source) -> Self {
        Self {
            interner: SymbolInterner::new(),
            source,
        }
    }

    pub fn intern_tok_str(&mut self, token: &Token) -> Symbol {
        self.interner.intern(token.view(&self.source.data))
    }

    pub fn lower(&mut self, tree: Program) -> HirProgram {
        let mut hir_tree = HirProgram { items: Vec::new() };

        for item in tree.items {
            hir_tree.items.push(self.lower_item(&item));
        }

        hir_tree
    }

    fn lower_item(&mut self, item: &Item) -> HirItem {
        let kind = match &item.kind {
            ItemKind::Fn(func) => HirItemKind::Fn(Box::new(self.lower_fn(&func))),
            _ => todo!(),
        };

        HirItem {
            kind,
            span: item.span,
        }
    }

    fn lower_fn(&mut self, func: &Fn) -> HirFn {
        HirFn {
            ident: self.lower_raw_ident(&func.ident),
            params: self.lower_fn_params(&func.params),
            ret_ty: None, // TODO
            // ret_ty: self.lower_ty(func.ret_ty),
            span: func.span(),
        }
    }

    pub fn lower_fn_params(&mut self, params: &FnParamList) -> HirFnParamList {
        let mut data = Vec::new();

        for param in &params.list {
            data.push(HirFnParam {
                ident: self.lower_raw_ident(&param.ident),
                // TODO: ty: None,
                span: param.span(),
            })
        }

        HirFnParamList {
            list: data,
            span: params.span,
        }
    }

    pub fn lower_expr(&mut self, expr: &Expr) -> HirExpr {
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
            kind,
            span: expr.span,
            eval: expr.eval,
        }
    }

    pub fn lower_literal(&mut self, lit: &Token) -> HirLiteral {
        todo!()
    }

    pub fn lower_binary(
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

    pub fn lower_unary(&mut self, expr: &Unary) -> HirUnary {
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

    pub fn lower_ty(&mut self, ty: &Ty) -> HirTy {
        let kind = match &ty.kind {
            TyKind::Path(path) => HirTyKind::Path(Box::new(self.lower_path(&path))),
            TyKind::Array(arr) => HirTyKind::Array(Box::new(self.lower_array(&arr))),
            TyKind::Unit(span) => HirTyKind::Unit(*span),
        };

        HirTy {
            kind,
            span: ty.span,
        }
    }

    pub fn lower_array(&mut self, arr: &Array) -> HirArray {
        HirArray {
            ty: Box::new(self.lower_ty(&arr.ty)),
            size: Box::new(self.lower_expr(&arr.size)),
            span: arr.span,
        }
    }

    pub fn lower_path(&mut self, path: &Path) -> HirPath {
        HirPath {
            segments: path
                .segments
                .iter()
                .map(|segment| self.lower_ident(segment))
                .collect(),
            span: path.span,
        }
    }

    pub fn lower_ident(&mut self, ident: &Identifier) -> HirIdent {
        HirIdent {
            ident: self.lower_raw_ident(&ident.ident),
            arguments: ident
                .arguments
                .as_ref()
                .map(|args| self.lower_ident_args(&args)),
            span: ident.span,
        }
    }

    pub fn lower_ident_args(&mut self, arguments: &IdentifierArguments) -> HirIdentArguments {
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

    pub fn lower_raw_ident(&mut self, token: &Token) -> HirRawIdent {
        HirRawIdent {
            ident: self.intern_tok_str(token),
            span: token.span,
        }
    }
}
