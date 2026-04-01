use hycc_ast::Program;
use hycc_collection::collector::Collector;
use hycc_diagnostic::{
    DiagnosticContext,
    reporter::{CLIReporter, DiagnosticReporter},
};
use hycc_hir::{builder::HirBuilder, program::HirProgram};
use hycc_parser::{
    lexer::Lexer,
    parser::{Parser, diag_ctx::ParserDiagCtx},
};
use hycc_source::Source;
use hycc_symbol::SymbolInterner;
use hycc_util::ternary;

// use hycc_collection::collector::SymCollector;

use crate::session::Session;

pub fn start(root_path: &str) {
    let mut session = Session::new(Source::new(root_path));
    compile(&mut session);

    let reporter = CLIReporter::new(&session.dctx, &session.source_registry);
    reporter.report();
}

// TODO: analyze all sources starting from the root
pub fn analyze_source(session: &mut Session) -> Option<Program> {
    let mut lexer = Lexer::new(session.source_registry.root(), &mut session.dctx);
    let tok_stream = lexer.tokenize();
    if session.dctx.error_occurred() {
        return None;
    }

    let mut parser = Parser::new(
        &session.source_registry.root(),
        ParserDiagCtx::new(session.dctx.data_mut()),
        tok_stream,
    );

    let program = parser.parse();
    ternary!(parser.dctx.error_occurred(), None, Some(program))
}

// TODO: lower the trees of other sources other than the root
pub fn lower_ast_to_hir(session: &Session, tree: Program) -> HirProgram {
    let mut hir_builder = HirBuilder::new(session.source_registry.root());
    hir_builder.lower(tree)
}

pub fn compile(session: &mut Session) {
    let Some(tree) = analyze_source(session) else {
        return;
    };

    let hir = lower_ast_to_hir(session, tree);
    dbg!(hir);

    // let mut interner = SymbolInterner::new();
    // let mut collector = Collector::new(&mut interner, &session.source_registry.root());

    // collector.collect(tree);
}
