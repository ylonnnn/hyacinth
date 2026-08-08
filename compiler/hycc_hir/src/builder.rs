use std::rc::Rc;

use hycc_ast::{
    Block, Expr, ExprKind, Identifier, Item, ItemKind, Path, Stmt, StmtKind, Ty, TyKind,
    expr::{
        AnonFn, AnonFnParamList, ArrayExpr, CallArguments, FieldAccess, FnCall, IfExpr, MethodCall,
        RefExpr, StructExpr, StructExprField, TupleExpr, Unary,
    },
    generic::{GenericParam, GenericParamList},
    item::{
        Extend, Fn, FnParamList, FnSig, Petal, PetalKind, Proto, ProtoItem, ProtoItemAssocFnKind,
        Refer, ReferTarget, ReferTargetKind, Struct, StructFieldList, VarDecl,
    },
    path::{IdentifierArgument, IdentifierArguments},
    stmt::{PassStmt, RetStmt},
    token::{Token, TokenKind},
    ty::{Array, FnTy, Ref, Slice, Tuple},
};
use hycc_const::{constant::ConstKind, table::ConstTable};
use hycc_source::SourceRegistry;
use hycc_symbol::{Symbol, SymbolInterner};
use hycc_util::{bug, digit_value, ternary};

use crate::{
    HirId, HirNode, HirTable,
    block::HirBlock,
    expr::{
        BinaryOp, HirAnonFn, HirAnonFnParam, HirAnonFnParamList, HirArrayExpr, HirCallArguments,
        HirExpr, HirExprKind, HirFieldAccess, HirFieldAccessField, HirFieldAccessFieldKind,
        HirFnCall, HirIfExpr, HirLiteral, HirMethodCall, HirRefExpr, HirStructExpr,
        HirStructExprField, HirTupleExpr, HirUnary, UnaryOp,
    },
    generic::{HirGenericParam, HirGenericParamList},
    item::{
        HirExtend, HirFn, HirFnParam, HirFnParamList, HirFnSig, HirItem, HirItemKind, HirItemLevel,
        HirPetal, HirPetalKind, HirProto, HirProtoItem, HirProtoItemAssocFnKind, HirRefer,
        HirReferTarget, HirReferTargetKind, HirStruct, HirStructField, HirStructFieldList,
        HirVarDecl,
    },
    path::{HirIdent, HirIdentArgument, HirIdentArguments, HirPath, HirRawIdent},
    stmt::{HirPassStmt, HirRetStmt, HirStmt, HirStmtKind},
    ty::{HirArray, HirFnTy, HirRef, HirSlice, HirTuple, HirTy, HirTyKind},
};

#[derive(Debug)]
pub struct HirBuilder<'i, 's, 't, 'h, 'c>
where
    'h: 't,
{
    interner: &'i mut SymbolInterner,
    registry: &'s SourceRegistry,
    hir_table: &'t HirTable<'h>,
    const_table: &'c mut ConstTable,
}

impl<'i, 's, 't, 'h, 'c> HirBuilder<'i, 's, 't, 'h, 'c> {
    pub fn new(
        interner: &'i mut SymbolInterner,
        source: &'s SourceRegistry,
        hir_table: &'t HirTable<'h>,
        const_table: &'c mut ConstTable,
    ) -> Self {
        Self {
            interner,
            registry: source,
            hir_table,
            const_table,
        }
    }

    pub fn intern_str(&mut self, s: &str) -> Symbol {
        self.interner.intern(s)
    }

    pub fn intern_tok_str(&mut self, token: &Token) -> Symbol {
        let source = self.registry.get(token.span.src_id);
        self.intern_str(token.view(&source.data))
    }

    pub fn lower(&mut self, tree: Petal) -> &'h HirItem<'h> {
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
            HirItemLevel::Top,
            tree.span,
        ))) {
            ternary!(
                matches!(&item.kind, HirItemKind::Petal(_)),
                item,
                unreachable!()
            )
        } else {
            unreachable!()
        }
    }

    fn lower_item(&mut self, item: &Item) -> &'h HirItem<'h> {
        let kind = match &item.kind {
            ItemKind::Refer(refer) => HirItemKind::Refer(Box::new(self.lower_refer(&refer))),
            ItemKind::Petal(petal) => HirItemKind::Petal(Box::new(self.lower_petal(&petal))),
            ItemKind::Proto(proto) => HirItemKind::Proto(Box::new(self.lower_proto(&proto))),
            ItemKind::Extend(extend) => HirItemKind::Extend(Box::new(self.lower_extend(&extend))),
            ItemKind::Struct(strct) => HirItemKind::Struct(Box::new(self.lower_struct(&strct))),
            ItemKind::Fn(func) => HirItemKind::Fn(Box::new(self.lower_fn(&func))),
            ItemKind::VarDecl(decl) => HirItemKind::VarDecl(Box::new(self.lower_var_decl(&decl))),
        };

        let mut hir_item = HirItem::new(kind, item.level, item.span);
        hir_item.accessibility = item.accessibility;

        if let HirNode::Item(item) = self.hir_table.add(HirNode::Item(hir_item)) {
            item
        } else {
            unreachable!()
        }
    }

    fn lower_refer(&mut self, refer: &Refer) -> HirRefer<'h> {
        HirRefer {
            target: self.lower_refer_target(&refer.target),
            span: refer.span,
        }
    }

    fn lower_refer_target(&mut self, target: &ReferTarget) -> HirReferTarget<'h> {
        let kind = match &target.kind {
            ReferTargetKind::Child(alias) => {
                HirReferTargetKind::Child(alias.as_ref().map(|alias| self.intern_tok_str(&alias)))
            }
            ReferTargetKind::Parent(children) => HirReferTargetKind::Parent(
                children
                    .iter()
                    .map(|child| self.lower_refer_target(&child))
                    .collect::<Vec<_>>(),
            ),
        };

        HirReferTarget {
            kind,
            symbol: self.lower_ident(&target.symbol),
            span: target.span,
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

    fn lower_proto(&mut self, proto: &Proto) -> HirProto<'h> {
        HirProto {
            ident: self.lower_raw_ident(&proto.ident),
            items: proto
                .items
                .iter()
                .map(|item| self.lower_proto_item(&item))
                .collect::<Vec<_>>(),
            span: proto.span,
        }
    }

    fn lower_proto_item(&mut self, item: &ProtoItem) -> HirProtoItem<'h> {
        match &item {
            // ProtoItem::AssocTy(ty) => HirProtoItem::AssocTy(self.lower_ty(&ty)),
            ProtoItem::AssocConst(decl) => HirProtoItem::AssocConst(self.lower_item(&decl)),
            ProtoItem::AssocFn(kind) => HirProtoItem::AssocFn(match &kind {
                ProtoItemAssocFnKind::Sig(sig) => {
                    HirProtoItemAssocFnKind::Sig(self.lower_fn_sig(&sig))
                }

                ProtoItemAssocFnKind::Impl(func) => {
                    HirProtoItemAssocFnKind::Impl(self.lower_item(&func))
                }
            }),
        }
    }

    fn lower_extend(&mut self, extend: &Extend) -> HirExtend<'h> {
        HirExtend {
            target: self.lower_ty(&extend.target),
            generic_params: extend
                .generic_params
                .as_ref()
                .map(|generic_params| self.lower_generic_params(generic_params)),
            items: extend
                .items
                .iter()
                .map(|item| self.lower_item(&item))
                .collect::<Vec<_>>(),
            span: extend.span(),
        }
    }

    fn lower_struct(&mut self, strct: &Struct) -> HirStruct<'h> {
        HirStruct {
            ident: self.lower_raw_ident(&strct.ident),
            generic_params: strct
                .generic_params
                .as_ref()
                .map(|generic_params| self.lower_generic_params(&generic_params)),
            fields: self.lower_struct_fields(&strct.fields),
        }
    }

    fn lower_struct_fields(&mut self, fields: &StructFieldList) -> HirStructFieldList<'h> {
        let mut data = Vec::new();

        for field in &fields.list {
            if let HirNode::StructField(field) =
                self.hir_table
                    .add(HirNode::StructField(HirStructField::<'h>::new(
                        self.lower_raw_ident(&field.ident),
                        self.lower_ty(&field.ty),
                        field.accessibility,
                        field.ident.span.merge(field.ty.span),
                    )))
            {
                data.push(field);
            } else {
                unreachable!();
            }
        }

        HirStructFieldList {
            list: data,
            span: fields.span,
        }
    }

    fn lower_fn_sig(&mut self, sig: &FnSig) -> HirFnSig<'h> {
        HirFnSig {
            ident: self.lower_raw_ident(&sig.ident),
            generic_params: sig
                .generic_params
                .as_ref()
                .map(|generic_params| self.lower_generic_params(&generic_params)),
            params: self.lower_fn_params(&sig.params),
            ret_ty: sig.ret_ty.as_ref().map(|ret_ty| self.lower_ty(ret_ty)),
        }
    }

    fn lower_fn(&mut self, func: &Fn) -> HirFn<'h> {
        HirFn {
            sig: self.lower_fn_sig(&func.sig),
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
            mutability: decl.mutability,
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
            StmtKind::Ret(ret) => HirStmtKind::Ret(Box::new(self.lower_ret_stmt(&ret))),
            StmtKind::Pass(pass) => HirStmtKind::Pass(Box::new(self.lower_pass_stmt(&pass))),
            StmtKind::Item(item) => HirStmtKind::Item(self.lower_item(item)),
            StmtKind::Expr(expr) => HirStmtKind::Expr(self.lower_expr(expr)),
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

    fn lower_ret_stmt(&mut self, ret: &RetStmt) -> HirRetStmt<'h> {
        HirRetStmt {
            value: ret.value.as_ref().map(|value| self.lower_expr(&value)),
            span: ret.span,
        }
    }

    fn lower_pass_stmt(&mut self, pass: &PassStmt) -> HirPassStmt<'h> {
        HirPassStmt {
            value: pass.value.as_ref().map(|value| self.lower_expr(&value)),
            label: pass
                .label
                .as_ref()
                .map(|label| self.lower_raw_ident(&label)),
            span: pass.span,
        }
    }

    fn lower_expr(&mut self, expr: &Expr) -> &'h HirExpr<'h> {
        let kind = match &expr.kind {
            ExprKind::Path(path) => HirExprKind::Path(self.lower_path(&path)),
            ExprKind::RefExpr(reference) => {
                HirExprKind::RefExpr(Box::new(self.lower_ref_expr(&reference)))
            }

            ExprKind::Literal(lit) => HirExprKind::Literal(Box::new(self.lower_literal(lit))),
            ExprKind::Binary(op, left, right) => {
                let (op, left, right) = self.lower_binary(op, left, right);
                HirExprKind::Binary(op, left, right)
            }

            ExprKind::Unary(unary) => HirExprKind::Unary(Box::new(self.lower_unary(unary))),
            ExprKind::Assign(assignee, expr) => {
                HirExprKind::Assign(self.lower_expr(assignee), self.lower_expr(expr))
            }

            ExprKind::Block(block) => HirExprKind::Block(self.lower_block(&block)),

            ExprKind::Array(array) => HirExprKind::Array(Box::new(self.lower_array_expr(&array))),
            ExprKind::Tuple(tup) => HirExprKind::Tuple(Box::new(self.lower_tuple_expr(&tup))),
            ExprKind::Struct(strct) => {
                HirExprKind::Struct(Box::new(self.lower_struct_expr(&strct)))
            }

            ExprKind::AnonFn(anfn) => HirExprKind::AnonFn(Box::new(self.lower_anon_fn(&anfn))),

            ExprKind::FnCall(call) => HirExprKind::FnCall(Box::new(self.lower_fn_call(&call))),

            ExprKind::FieldAccess(access) => {
                HirExprKind::FieldAccess(Box::new(self.lower_field_access(&access)))
            }

            ExprKind::MethodCall(call) => {
                HirExprKind::MethodCall(Box::new(self.lower_method_call(&call)))
            }

            ExprKind::If(ite) => HirExprKind::If(Box::new(self.lower_if_expr(&ite))),
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

    fn lower_ref_expr(&mut self, reference: &RefExpr) -> HirRefExpr<'h> {
        HirRefExpr {
            expr: self.lower_expr(&reference.expr),
            mutability: reference.mutability,
            span: reference.span,
        }
    }

    fn lower_literal(&mut self, lit: &Token) -> HirLiteral {
        let source = self.registry.get(lit.span.src_id);
        let view = lit.view(&source.data);

        let const_kind = match &lit.kind {
            TokenKind::Int { base } | TokenKind::Float { base } => {
                let view = ternary!(*base != 10, &view[2..], view);
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

                    ConstKind::float(val)
                } else {
                    ConstKind::Int(val as u64)
                }
            }

            TokenKind::Bool => ConstKind::Bool(view == "true"),
            TokenKind::Char { .. } => ConstKind::Char(view.as_bytes()[0]), // TODO: maybe
            // add support to
            // larger sized
            // characters
            TokenKind::String { .. } => ConstKind::String(Rc::from(view)),

            _ => unreachable!(),
        };

        HirLiteral(self.const_table.intern(const_kind))
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
                    TokenKind::Minus => UnaryOp::Negate,
                    TokenKind::Bang => UnaryOp::Not,
                    TokenKind::Star => UnaryOp::Deref,

                    _ => UnaryOp::Nop,
                },
                self.lower_expr(&expr),
            ),
            Unary::Post(op, expr) => HirUnary::Post(
                match op.kind {
                    _ => UnaryOp::Nop,
                },
                self.lower_expr(&expr),
            ),
        }
    }

    fn lower_call_arguments(&mut self, call_args: &CallArguments) -> HirCallArguments<'h> {
        HirCallArguments {
            data: call_args
                .data
                .iter()
                .map(|arg| self.lower_expr(arg))
                .collect(),
            span: call_args.span,
        }
    }

    fn lower_array_expr(&mut self, expr: &ArrayExpr) -> HirArrayExpr<'h> {
        HirArrayExpr {
            elements: expr
                .elements
                .iter()
                .map(|el| self.lower_expr(&el))
                .collect(),
            span: expr.span,
        }
    }

    fn lower_tuple_expr(&mut self, expr: &TupleExpr) -> HirTupleExpr<'h> {
        HirTupleExpr {
            elements: expr
                .elements
                .iter()
                .map(|el| self.lower_expr(&el))
                .collect(),
            span: expr.span,
        }
    }

    fn lower_struct_expr(&mut self, expr: &StructExpr) -> HirStructExpr<'h> {
        HirStructExpr {
            path: self.lower_path(&expr.path),
            fields: self.lower_struct_expr_fields(&expr.fields),
            span: expr.span,
        }
    }

    fn lower_struct_expr_fields(
        &mut self,
        fields: &[StructExprField],
    ) -> Vec<&'h HirStructExprField<'h>> {
        fields
            .iter()
            .map(|field| {
                if let HirNode::StructExprField(field) =
                    self.hir_table
                        .add(HirNode::StructExprField(HirStructExprField::new(
                            self.lower_raw_ident(&field.ident),
                            self.lower_expr(&field.val),
                        )))
                {
                    field
                } else {
                    unreachable!()
                }
            })
            .collect()
    }

    fn lower_anon_fn(&mut self, anfn: &AnonFn) -> HirAnonFn<'h> {
        HirAnonFn {
            params: self.lower_anon_fn_params(&anfn.params),
            ret_ty: anfn.ret_ty.as_ref().map(|ret_ty| self.lower_ty(ret_ty)),
            body: self.lower_block(&anfn.body),
            span: anfn.span,
        }
    }

    fn lower_anon_fn_params(&mut self, params: &AnonFnParamList) -> HirAnonFnParamList<'h> {
        let mut data = Vec::new();

        for param in &params.list {
            if let HirNode::AnonFnParam(param) =
                self.hir_table
                    .add(HirNode::AnonFnParam(HirAnonFnParam::<'h>::new(
                        self.lower_raw_ident(&param.ident),
                        param.ty.as_ref().map(|ty| self.lower_ty(&ty)),
                    )))
            {
                data.push(param);
            } else {
                unreachable!();
            }
        }

        HirAnonFnParamList {
            list: data,
            span: params.span,
        }
    }

    fn lower_fn_call(&mut self, call: &FnCall) -> HirFnCall<'h> {
        HirFnCall {
            callee: self.lower_expr(&call.callee),
            arguments: self.lower_call_arguments(&call.arguments),
        }
    }

    fn lower_field_access(&mut self, access: &FieldAccess) -> HirFieldAccess<'h> {
        let field = HirFieldAccessField {
            span: access.field.span,
            kind: match &access.field.kind {
                TokenKind::Ident(_) => {
                    HirFieldAccessFieldKind::Ident(self.lower_raw_ident(&access.field).ident)
                }
                TokenKind::Int { .. } => {
                    let lit = self.lower_literal(&access.field);
                    let ConstKind::Int(data) = self.const_table.get(lit.0) else {
                        bug!("fields can only be identifiers or integers")
                    };

                    HirFieldAccessFieldKind::Index(*data as usize)
                }

                _ => bug!("fields can only be identifiers or integers"),
            },
        };

        HirFieldAccess {
            leading: self.lower_expr(&access.leading),
            field,
        }
    }

    fn lower_method_call(&mut self, call: &MethodCall) -> HirMethodCall<'h> {
        HirMethodCall {
            receiver: self.lower_expr(&call.receiver),
            callee: self.lower_ident(&call.callee),
            arguments: self.lower_call_arguments(&call.arguments),
        }
    }

    fn lower_if_expr(&mut self, ite: &IfExpr) -> HirIfExpr<'h> {
        HirIfExpr {
            span: ite.span,
            cond: self.lower_expr(&ite.cond),
            consequent: self.lower_block(&ite.consequent),
            alternate: ite.alternate.as_ref().map(|alt| self.lower_block(&alt)),
        }
    }

    fn lower_ty(&mut self, ty: &Ty) -> &'h HirTy<'h> {
        let kind = match &ty.kind {
            TyKind::Unit(span) => HirTyKind::Unit(*span),
            TyKind::Path(path) => HirTyKind::Path(self.lower_path(&path)),
            TyKind::Ref(reference) => HirTyKind::Ref(Box::new(self.lower_ref(&reference))),

            TyKind::Array(arr) => HirTyKind::Array(Box::new(self.lower_array(&arr))),
            TyKind::Slice(slice) => HirTyKind::Slice(Box::new(self.lower_slice(&slice))),

            TyKind::Tuple(tup) => HirTyKind::Tuple(Box::new(self.lower_tuple(&tup))),
            TyKind::Fn(func) => HirTyKind::Fn(Box::new(self.lower_fn_ty(&func))),
        };

        if let HirNode::Ty(ty) = self.hir_table.add(HirNode::Ty(HirTy::new(kind, ty.span))) {
            ty
        } else {
            unreachable!()
        }
    }

    fn lower_ref(&mut self, reference: &Ref) -> HirRef<'h> {
        HirRef {
            ty: self.lower_ty(&reference.ty),
            mutability: reference.mutability,
            span: reference.span,
        }
    }

    fn lower_array(&mut self, arr: &Array) -> HirArray<'h> {
        HirArray {
            ty: self.lower_ty(&arr.ty),
            size: self.lower_expr(&arr.size),
            span: arr.span,
        }
    }

    fn lower_slice(&mut self, slice: &Slice) -> HirSlice<'h> {
        HirSlice {
            ty: self.lower_ty(&slice.ty),
            span: slice.span,
        }
    }

    fn lower_tuple(&mut self, tup: &Tuple) -> HirTuple<'h> {
        HirTuple {
            data: tup.data.iter().map(|el| self.lower_ty(&el)).collect(),
            span: tup.span,
        }
    }

    fn lower_fn_ty(&mut self, func: &FnTy) -> HirFnTy<'h> {
        HirFnTy {
            params: func
                .params
                .iter()
                .map(|param| self.lower_ty(&param))
                .collect(),
            ret_ty: func.ret_ty.as_ref().map(|ret_ty| self.lower_ty(&ret_ty)),
            span: func.span,
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

    fn lower_generic_params(
        &mut self,
        generic_params: &GenericParamList,
    ) -> HirGenericParamList<'h> {
        let mut data = Vec::new();

        for generic_param in &generic_params.list {
            if let HirNode::GenericParam(generic_param) =
                self.hir_table
                    .add(HirNode::GenericParam(HirGenericParam::<'h>::new(
                        self.lower_raw_ident(&generic_param.ident),
                        generic_param
                            .proto_reqs
                            .iter()
                            .map(|proto_req| self.lower_path(&proto_req))
                            .collect::<Vec<_>>(),
                        generic_param.kind,
                        generic_param.span(),
                    )))
            {
                data.push(generic_param);
            } else {
                unreachable!();
            }
        }

        HirGenericParamList {
            list: data,
            span: generic_params.span,
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
