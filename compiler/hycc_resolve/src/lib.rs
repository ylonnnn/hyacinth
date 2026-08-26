use hycc_hir::{
    def::{Binding, DefId, DefSpace, DefinitionTable},
    expr::HirExpr,
    generic::HirGenericParamKind,
    path::{HirIdent, HirIdentArgument, HirIdentArguments, HirPath},
    petal::PetalCtx,
    ty::HirTy,
};
use hycc_span::Span;
use hycc_symbol::Symbol;
use hycc_ty::{
    ctx::{TyCtx, TyId},
    extension::ExtensionId,
    ty::{GenericArg, InferKind, Ty, TyKind},
};
use hycc_util::ternary;

use crate::diag::SymbolKind;

pub mod diag;

pub mod collector;
pub mod resolver;
pub mod ty_resolver;

pub trait ResolveExpr<T, E> {
    fn resolve_expr(&mut self, expr: &HirExpr) -> Result<T, E>;
}

pub trait ResolveTy<E> {
    fn resolve_ty(&mut self, ty: &HirTy) -> Result<TyId, E>;
}

pub trait ResolveIdentArgs<TEx, E>: ResolveExpr<TEx, E> + ResolveTy<E> {
    fn resolve_ident_args(&mut self, arguments: &HirIdentArguments) -> Result<Vec<GenericArg>, E> {
        arguments
            .data
            .iter()
            .map(|argument| match &argument {
                HirIdentArgument::Ty(ty) => self.resolve_ty(&ty).map(|ty_id| GenericArg::Ty(ty_id)),
                HirIdentArgument::Expr(expr) => todo!("resolve ident args expr"),
            })
            .collect::<Result<Vec<_>, E>>()
    }
}

pub trait InstantiateIdent<TEx, E>: ResolveIdentArgs<TEx, E> {
    fn definitions(&self) -> &DefinitionTable;
    fn definitions_mut(&mut self) -> &mut DefinitionTable;
    fn tctx(&mut self) -> &mut TyCtx;

    fn def_ty(&mut self, def_id: DefId, span: Span) -> Result<TyId, E>;

    fn generic_arg_arity_mismatch_error(&self, span: Span, expected: u8, received: u8) -> E;

    fn instantiate(
        &mut self,
        arg_frames: &mut Vec<Vec<GenericArg>>,
        ident: &HirIdent,
    ) -> Result<TyId, E> {
        let def_id = self.definitions().expect_def_id(ident.id);

        let generic_params = self
            .definitions()
            .get(def_id)
            .generic_params()
            .map_or_else(Vec::new, |params| {
                params.iter().cloned().collect::<Vec<_>>()
            });

        let generic_param_count = generic_params.len();

        let mut g_args = ident.arguments.as_ref().map_or_else(
            || Ok(Vec::new()),
            |arguments| -> Result<Vec<GenericArg>, E> {
                let g_args = self.resolve_ident_args(&arguments)?;
                let n = g_args.len();

                if n > generic_param_count {
                    return Err(self.generic_arg_arity_mismatch_error(
                        arguments.span,
                        generic_param_count as u8,
                        n as u8,
                    ));
                }

                Ok(g_args)
            },
        )?;

        for i in g_args.len()..generic_param_count {
            let gp_def_id = generic_params[i];
            let def = self.definitions().get(gp_def_id);
            let (gp_span, gp_def) = (def.span, def.kind.expect_generic_param());

            g_args.push(match &gp_def.kind {
                HirGenericParamKind::Ty => GenericArg::Ty(
                    self.tctx()
                        .make_inferred_ty(Span::default(), InferKind::Any),
                ),

                HirGenericParamKind::Const => todo!("const generic arg"),
            });
        }

        if !g_args.is_empty() {
            arg_frames.push(g_args);
        }

        let raw_ty_id = self.def_ty(def_id, ident.span)?;
        let ty_id = self.tctx().instantiate(
            raw_ty_id,
            &arg_frames
                .iter()
                .map(|args| args.as_slice())
                .collect::<Vec<_>>(),
        );

        self.tctx()
            .attach_to_hir(ident.id, Ty::new(ty_id, ident.span));

        Ok(ty_id)
    }
}

pub trait ResolvePath<TEx, E>: InstantiateIdent<TEx, E> {
    fn expected_space(&self) -> Option<DefSpace>;
    fn petal_ctx(&self) -> &PetalCtx;

    fn unrecognized_member_error(&self, span: Span, name: Symbol, ty_id: TyId) -> E;
    fn inaccessible_error(&self, span: Span, name: Symbol, kind: Option<SymbolKind>) -> E;
    fn multiple_assoc_item_matched_error(
        &self,
        span: Span,
        name: Symbol,
        matches: Vec<(ExtensionId, Binding)>,
    ) -> E;

    fn resolve_path(&mut self, path: &HirPath) -> Result<TyId, E> {
        let space = self
            .expected_space()
            .unwrap_or_else(|| panic!("expected an expected definition space"));

        let n = path.segments.len();
        let Some(res) = self.definitions().get_res(path.id) else {
            return Ok(self.tctx().make_error_ty());
        };

        let mut generic_args = Vec::new();

        let resolved_count = (n - res.unresolved);
        let mut prev_ty_id = self.instantiate(
            &mut generic_args,
            &path.segments[resolved_count.saturating_sub(1)],
        )?;

        for (i, ident) in path.segments[resolved_count..].iter().enumerate() {
            let space = ternary!(i == (n - resolved_count) - 1, space, DefSpace::Type);
            if self.definitions().get_def_id(ident.id).is_none() {
                let target = self.tctx().ext_target_kind_of(prev_ty_id);
                let assoc_items =
                    self.tctx()
                        .ext_table
                        .get_assoc_items(target, space, ident.ident.ident);

                if assoc_items.is_empty() {
                    return Err(self.unrecognized_member_error(
                        ident.span,
                        ident.ident.ident,
                        prev_ty_id,
                    ));
                };

                let (_, assoc_item) = &assoc_items[0];
                self.definitions_mut()
                    .define_id_hir(ident.id, assoc_item.def_id);

                if assoc_items.len() > 1 {
                    return Err(self.multiple_assoc_item_matched_error(
                        ident.span,
                        ident.ident.ident,
                        assoc_items,
                    ));
                }

                let def = self.definitions().get(assoc_item.def_id);
                if !self.petal_ctx().accessible(&def) {
                    return Err(self.inaccessible_error(
                        ident.span,
                        def.name,
                        Some(SymbolKind::AssocItem),
                    ));
                }
            };

            prev_ty_id = self.instantiate(&mut generic_args, &ident)?;
        }

        let definitions = self.definitions_mut();
        definitions.define_id_hir(
            path.id,
            definitions.expect_def_id(path.segments.last().unwrap().id),
        );

        self.tctx()
            .attach_to_hir(path.id, Ty::new(prev_ty_id, path.span));

        Ok(prev_ty_id)
    }
}
