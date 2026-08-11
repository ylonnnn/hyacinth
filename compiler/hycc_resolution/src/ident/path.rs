use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirId,
    def::{
        Binding, DefAccessibility, DefId, DefNodeResolution, DefPubAccessibilityKind, DefResKind,
        DefResolution, DefSpace,
    },
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
        let space = self
            .expected_space
            .expect("expected definition space must exist");

        let n = path.segments.len();
        let (mut resolution, mut resolved) = (None, 0_usize);

        for (i, segment) in path.segments.iter().enumerate() {
            let is_last = i == n - 1;

            let res = self.expect_space(
                ternary!(is_last, space, DefSpace::Type),
                |s| -> ResolveResult<Option<DefResolution>> {
                    s.resolve_ident(&segment, resolution)
                },
            )?;

            if let Some(res) = res {
                resolution.replace(res);
                resolved += 1;
            }

            if is_last {
                self.collector
                    .definitions
                    .get_def_id(segment.id)
                    .map(|def_id| self.collector.definitions.define_id_hir(path.id, def_id));
            }
        }

        self.collector.definitions.attach_res(
            path.id,
            DefNodeResolution {
                base: resolution.unwrap(),
                unresolved: n - resolved,
            },
        );

        Ok(())
    }

    // std::vec::Vec<i32>::ElementType
    // std - (Default) petal (Resolution::Petal(DefId))
    // vec - (Petal-based (Scope)) petal (Resolution::Petal(DefId))
    // Vec<i32> - (Petal-based (Scope)) type (Resolution::Ty(HirId))
    // ElementType - (Type-based (`extend`-based lookup)) type (Resolution::Ty(HirId))

    pub(crate) fn resolve_ident(
        &mut self,
        ident: &HirIdent,
        resolution: Option<DefResolution>,
    ) -> ResolveResult<Option<DefResolution>> {
        let name = ident.ident.ident;
        if let Some(arguments) = &ident.arguments {
            for argument in &arguments.data {
                let res = match &argument {
                    HirIdentArgument::Expr(expr) => {
                        self.expect_space(DefSpace::Value, |s| s.resolve_expr(&expr))
                    }

                    HirIdentArgument::Ty(ty) => {
                        self.expect_space(DefSpace::Type, |s| s.resolve_ty(&ty))
                    }
                };

                if let Err(Some(diag)) = res {
                    self.dctx.add(diag);
                }
            }
        }

        if !matches!(&resolution, Some(DefResolution::Petal(_)) | None) {
            return Ok(None);
        }

        let Some(&Binding { def_id, .. }) = resolution.map_or_else(
            || {
                self.get_binding(self.expected_space, name)
                    .map(|(binding, _)| binding)
            },
            |res| match &res {
                DefResolution::Petal(def_id) => self
                    .collector
                    .scope_ctx
                    .expect_def_scope(*def_id)
                    .get(self.expected_space, name),
                _ => bug!(
                    "cannot resolve identifiers from the given resolution {:?}",
                    res
                ),
            },
        ) else {
            return Err(Some(ResolverDiag::error(
                ident.span,
                ResolverDiagErrorKind::UnrecognizedSymbol(name, self.expected_space),
            )));
        };

        self.collector.definitions.define_id_hir(ident.id, def_id);

        let def = self.collector.definitions.get(def_id);
        let res_kind = def.kind.res_kind();

        if !self.collector.petal_ctx.accessible(&def) {
            Err(Some(ResolverDiag::error(
                ident.span,
                ResolverDiagErrorKind::InaccessibleSymbol(name),
            )))?
        }

        Ok(Some(match &res_kind {
            DefResKind::Petal => DefResolution::Petal(def_id),
            DefResKind::Ty => DefResolution::Ty(def_id),
            DefResKind::Value => DefResolution::Value(def_id),
        }))
    }

    pub(crate) fn resolve_ident1(
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

        if let Some(arguments) = &ident.arguments {
            for argument in &arguments.data {
                let res = match &argument {
                    HirIdentArgument::Expr(expr) => {
                        self.expect_space(DefSpace::Value, |s| s.resolve_expr(&expr))
                    }

                    HirIdentArgument::Ty(ty) => {
                        self.expect_space(DefSpace::Type, |s| s.resolve_ty(&ty))
                    }
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

        self.collector.definitions.define_id_hir(ident.id, def_id);

        Ok(def_id)
    }
}
