use hycc_diagnostic::DiagnosticContext;
use hycc_hir::block::HirBlock;

use crate::inferer::{InferResult, TyInferer};

impl<'t, 'd, 'r> TyInferer<'t, 'd, 'r> {
    pub(crate) fn infer_block(&mut self, block: &HirBlock) -> InferResult {
        for stmt in &block.stmts {
            if let Err(Some(diag)) = self.infer_stmt(&stmt) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }
}
