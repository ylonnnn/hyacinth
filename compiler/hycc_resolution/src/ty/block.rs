use hycc_diagnostic::DiagnosticContext;
use hycc_hir::block::HirBlock;

use crate::{ResolveResult, ty::resolver::TyResolver};

impl<'t, 'd, 's, 'h> TyResolver<'t, 'd, 's, 'h> {
    pub(crate) fn resolve_block(&mut self, block: &HirBlock) -> ResolveResult {
        for stmt in &block.stmts {
            if let Err(Some(diag)) = self.resolve_stmt(&stmt) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }
}
