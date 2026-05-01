use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    def::DefSpace,
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

        let n = path.segments.len();
        for (i, segment) in path.segments.iter().enumerate() {
            let is_last = i == n - 1;

            self.expect_space(ternary!(is_last, space, DefSpace::Type), |s| {
                if let Err(Some(diag)) = s.resolve_ident(&segment) {
                    s.dctx.add(diag);
                }
            });

            if is_last {
                if let Some(def_id) = self.resolved.get(&segment.id) {
                    self.resolved.insert(path.id, *def_id);
                }
            }
        }

        Ok(())
    }

    pub(crate) fn resolve_ident(&mut self, ident: &HirIdent) -> ResolveResult {
        let Some(space) = self.expected_space else {
            bug!("expected space must not be `None`");
        };

        let Some(def_id) = self.get_def_id(space, ident.ident.ident) else {
            return Err(Some(ResolverDiag::error(
                ident.span,
                ResolverDiagErrorKind::UnrecognizedSymbol(ident.ident.ident, space),
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

        self.resolved.insert(ident.id, def_id);
        Ok(())
    }
}
