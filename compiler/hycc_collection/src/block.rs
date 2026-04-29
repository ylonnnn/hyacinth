use hycc_diagnostic::DiagnosticContext;
use hycc_hir::block::HirBlock;
use hycc_scope::Scope;

use crate::collector::{CollectResult, CollectionLevel, Collector};

impl<'t, 'h> Collector<'t, 'h> {
    pub(crate) fn collect_block(&mut self, block: &HirBlock) -> CollectResult {
        let scope_id = self.scope_ctx.attach(block.id, Scope::new());

        self.enter_scope(scope_id, CollectionLevel::Local, |s| -> CollectResult {
            for stmt in &block.stmts {
                if let Err(Some(diag)) = s.collect_stmt(&stmt) {
                    s.dctx.add(diag);
                }
            }

            Ok(())
        })
    }
}
