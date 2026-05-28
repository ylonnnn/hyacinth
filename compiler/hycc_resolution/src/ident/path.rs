use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    def::{DefId, DefSpace},
    path::{HirIdent, HirIdentArgument, HirPath},
};
use hycc_util::{bug, ternary};

use crate::{
    ResolveResult,
    diag::{ResolverDiag, ResolverDiagErrorKind},
    ident::resolver::Resolver,
};

impl<'c> Resolver<'c> {
    pub(crate) fn resolve_path(&mut self, path: &HirPath) -> ResolveResult {
        let Some(space) = self.expected_space else {
            bug!("expected definition space must exist!")
        };

        let top_id = self.collector.scope_ctx.top_id();
        let n = path.segments.len();

        for (i, segment) in path.segments.iter().enumerate() {
            let is_last = i == n - 1;

            self.expect_space(
                ternary!(is_last, space, DefSpace::Type),
                |s| -> ResolveResult {
                    let def_id = s.resolve_ident(&segment)?;
                    Ok(s.collector
                        .scope_ctx
                        .get_id_from_def(def_id)
                        .map(|scope_id| s.collector.scope_ctx.push_id(scope_id))
                        .unwrap_or(()))
                },
            )?;

            if is_last {
                let def_id = self.collector.definitions.get_def_id(segment.id).cloned();
                def_id.map(|def_id| self.collector.definitions.define_id_hir(path.id, def_id));
            }
        }

        while top_id != self.collector.scope_ctx.top_id() {
            self.collector.scope_ctx.pop();
        }

        Ok(())
    }

    pub(crate) fn resolve_ident(&mut self, ident: &HirIdent) -> ResolveResult<DefId> {
        let Some(space) = self.expected_space else {
            bug!("expected space must not be `None`");
        };

        let Some(def_id) = self.get_def_id(Some(space), ident.ident.ident) else {
            return Err(Some(ResolverDiag::error(
                ident.span,
                ResolverDiagErrorKind::UnrecognizedSymbol(ident.ident.ident, Some(space)),
            )));
        };

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

        self.collector.definitions.define_id_hir(ident.id, def_id);
        Ok(def_id)
    }
}
