use hycc_ast::Program;
use hycc_collection::collector::Collector;
use hycc_diagnostic::{
    DiagnosticContext,
    reporter::{CLIReporter, DiagnosticReporter},
};
use hycc_hir::{builder::HirBuilder, program::HirProgram};
use hycc_parser::{lexer::Lexer, parser::Parser};
use hycc_source::Source;
use hycc_util::ternary;

use crate::session::Session;

pub fn start(root_path: &str) {
    let mut session = Session::new(Source::new(root_path));
    compile(&mut session);

    let reporter = CLIReporter::new(&session.dctx, &session.source_registry);
    reporter.report();
}

// TODO: analyze all sources starting from the root
pub fn analyze_source(session: &mut Session) -> Option<Program> {
    let mut lexer = Lexer::new(session.source_registry.root());
    let tok_stream = lexer.tokenize();

    lexer.dctx.emit(&mut session.dctx, &session.source_registry);
    if session.dctx.error_occurred() {
        return None;
    }

    let mut parser = Parser::new(&session.source_registry.root(), tok_stream);
    parser
        .dctx
        .emit(&mut session.dctx, &session.source_registry);

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
    let mut collector = Collector::new(&mut session.dctx);

    collector.collect(hir);
}
