use hycc_diagnostic::DiagnosticContext;
use hycc_hir::block::HirBlock;
use hycc_util::bug;

use crate::{ResolveResult, ident::resolver::Resolver};

impl<'s, 'd> Resolver<'s, 'd> {
    pub(crate) fn resolve_block(&mut self, block: &HirBlock) -> ResolveResult {
        let Some(scope_id) = self.scope_ctx.get_id(block.id) else {
            bug!("block {:?} does not have an attached scope!", block.id)
        };

        self.enter_scope(scope_id, |s| {
            for stmt in &block.stmts {
                if let Err(Some(diag)) = s.resolve_stmt(&stmt) {
                    s.dctx.add(diag);
                }
            }

            Ok(())
        })
    }
}
