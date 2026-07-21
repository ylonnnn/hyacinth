use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{block::HirBlock, scope::Scope};

use crate::{ResolveResult, ident::resolver::Resolver};

impl<'c, 'i> Resolver<'c, 'i> {
    pub(crate) fn resolve_block(&mut self, block: &HirBlock) -> ResolveResult {
        let scope_id = self.collector.scope_ctx.attach(block.id, Scope::new());

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
