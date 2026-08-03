use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    def::{DefAccessibility, DefId, DefPubAccessibilityKind, DefSpace},
    path::{HirIdent, HirIdentArgument, HirPath},
    petal::PetalRelationship,
    scope::ScopeId,
};
use hycc_ty::ty::{GenericArg, Ty};
use hycc_util::{bug, ternary};

use crate::{
    ResolveResult,
    diag::{ResolverDiag, ResolverDiagErrorKind},
    ident::resolver::Resolver,
};

impl<'c, 'i, 'h> Resolver<'c, 'i, 'h> {
    pub(crate) fn resolve_path(&mut self, path: &HirPath) -> ResolveResult {
        let Some(space) = self.expected_space else {
            bug!("expected definition space must exist!")
        };

        // let top_id = self.collector.scope_ctx.top_id();

        let n = path.segments.len();
        // let segment = path.segments.first().unwrap();
        let mut segment_scope = None;

        for (i, segment) in path.segments.iter().enumerate() {
            let is_last = i == n - 1;
            // let (is_first, is_last) = (i == 0, i == n - 1);

            segment_scope = self.expect_space(
                ternary!(is_last, space, DefSpace::Type),
                |s| -> ResolveResult<Option<ScopeId>> {
                    let def_id = s.resolve_ident(&segment, segment_scope)?;

                    Ok(ternary!(
                        is_last,
                        None,
                        s.collector.scope_ctx.get_id_from_def(def_id)
                    ))
                },
            )?;

            if is_last {
                self.collector
                    .definitions
                    .get_def_id(segment.id)
                    .map(|def_id| self.collector.definitions.define_id_hir(path.id, def_id));
            }
        }

        // while top_id != self.collector.scope_ctx.top_id() {
        //     self.collector.scope_ctx.pop();
        // }

        Ok(())
    }

    pub(crate) fn resolve_ident(
        &mut self,
        ident: &HirIdent,
        lookup_scope: Option<ScopeId>,
    ) -> ResolveResult<DefId> {
        let name = ident.ident.ident;
        let binding = if let Some(scope) = lookup_scope {
            self.collector
                .scope_ctx
                .get(scope)
                .get(self.expected_space, name)
                .map(|binding| (binding, scope))
        } else {
            self.get_binding(self.expected_space, name)
        };

        let err = || {
            Err(Some(ResolverDiag::error(
                ident.span,
                ResolverDiagErrorKind::UnrecognizedSymbol(name, self.expected_space),
            )))
        };

        let Some((binding, _)) = binding else { err()? };

        let definition = self.collector.definitions.get(binding.def_id);
        if !self.collector.petal_ctx.accessible(&definition) {
            Err(Some(ResolverDiag::error(
                ident.span,
                ResolverDiagErrorKind::InaccessibleSymbol(name),
            )))?
        }

        let def_id = binding.def_id;
        // let mut generic_args = Vec::new();

        if let Some(arguments) = &ident.arguments {
            for argument in &arguments.data {
                let res = match &argument {
                    HirIdentArgument::Expr(expr) => {
                        // todo!("return GenericArg::Const or something")
                        self.expect_space(DefSpace::Value, |s| s.resolve_expr(&expr))
                    }

                    HirIdentArgument::Ty(ty) => {
                        self.expect_space(DefSpace::Type, |s| s.resolve_ty(&ty))
                    } // .and_then(|_| {
                      //     self.def_to_ty(def_id, ty.span)
                      //         .map(|ty_id| GenericArg::Ty(ty_id))
                      // }),
                };

                match res {
                    Ok(argument) => {
                        // generic_args.push(argument)
                    }
                    Err(diag) => {
                        diag.map(|diag| self.dctx.add(diag));
                    }
                }
            }
        }

        // let ty = self.def_to_ty(def_id, ident.span);
        // dbg!(&ty);
        // dbg!(self.collector.tctx.get(ty.unwrap()));
        // let ty_id = self.def_to_ty(def_id, ident.span)?;
        // let ty_id = self.collector.tctx.get_hir_ty_id(ident.id).unwrap();
        // let inst_ty_id = self.collector.tctx.instantiate(ty_id, generic_args.into());

        // self.collector
        //     .tctx
        //     .attach_to_hir(ident.id, Ty::new(inst_ty_id, ident.span));
        self.collector.definitions.define_id_hir(ident.id, def_id);

        Ok(def_id)
    }
}
