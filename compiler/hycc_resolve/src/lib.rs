use hycc_hir::{
    def::{DefId, DefinitionTable},
    expr::HirExpr,
    generic::HirGenericParamKind,
    path::{HirIdent, HirIdentArgument, HirIdentArguments},
    ty::HirTy,
};
use hycc_span::Span;
use hycc_ty::{
    context::{TyCtx, TyId},
    ty::{GenericArg, InferKind, Ty},
};

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
            .map(|params| params.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        let generic_param_count = generic_params.len();

        let mut g_args = if let Some(arguments) = &ident.arguments {
            let g_args = self.resolve_ident_args(&arguments)?;
            let n = g_args.len();

            if n > generic_param_count {
                return Err(self.generic_arg_arity_mismatch_error(
                    arguments.span,
                    generic_param_count as u8,
                    n as u8,
                ));
            }

            g_args
        } else {
            Vec::new()
        };

        for i in g_args.len()..generic_param_count {
            let gp_def_id = generic_params[i];
            let gp_def = self
                .definitions()
                .get(gp_def_id)
                .kind
                .expect_generic_param();

            g_args.push(match &gp_def.kind {
                HirGenericParamKind::Ty => {
                    GenericArg::Ty(self.tctx().make_inferred_ty(InferKind::Any))
                }

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
