use hycc_ast::Program;
use hycc_collection::{collector::Collector, diag::CollectorDiagDataCtx};
use hycc_diagnostic::{
    DiagnosticContext,
    reporter::{CLIReporter, DiagnosticReporter},
};
use hycc_hir::{builder::HirBuilder, program::HirProgram};
use hycc_parser::{
    lexer::{Lexer, diag::LexerDiagDataCtx},
    parser::{Parser, diag::ParserDiagDataCtx},
};
use hycc_source::Source;
use hycc_util::ternary;

use crate::session::Session;

pub fn invoke(root_path: &str) {
    let mut session = Session::new(Source::new(root_path));
    compile(&mut session);

    let reporter = CLIReporter::new(&session.dctx, &session.source_registry);
    reporter.report();
}

// TODO: analyze all sources starting from the root
pub fn analyze_source<'h>(session: &mut Session<'h>) -> Option<Program> {
    let mut lexer = Lexer::new(session.source_registry.root());
    let tok_stream = lexer.tokenize();

    lexer.dctx.emit(
        &mut session.dctx,
        LexerDiagDataCtx::new(&session.source_registry),
    );

    if session.dctx.error_occurred() {
        return None;
    }

    let mut parser = Parser::new(tok_stream);
    let program = parser.parse();

    parser.dctx.emit(
        &mut session.dctx,
        ParserDiagDataCtx::new(&session.source_registry),
    );

    if session.dctx.error_occurred() {
        return None;
    }

    ternary!(parser.dctx.error_occurred(), None, Some(program))
}

// TODO: lower the trees of other sources other than the root
pub fn lower_ast_to_hir<'h>(session: &mut Session<'h>, tree: Program) -> &'h HirProgram<'h> {
    let mut hir_builder = HirBuilder::new(
        &mut session.interner,
        session.source_registry.root(),
        &session.hir_table,
    );

    hir_builder.lower(tree)
}

pub fn compile<'h>(session: &mut Session<'h>) {
    let Some(tree) = analyze_source(session) else {
        return;
    };

    let hir = lower_ast_to_hir(session, tree);
    let mut collector = Collector::new(&session.hir_table);

    collector.collect(hir);

    let (definitions, scope_ctx) = (&collector.definitions, &collector.scope_ctx);

    collector.dctx.emit(
        &mut session.dctx,
        CollectorDiagDataCtx::new(
            &session.interner,
            &session.hir_table,
            &definitions,
            &scope_ctx,
        ),
    );
}
