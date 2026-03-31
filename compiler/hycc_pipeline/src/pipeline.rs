use hycc_ast::Program;
use hycc_diagnostic::{
    DiagnosticContext,
    reporter::{CLIReporter, DiagnosticReporter},
};
use hycc_parser::{
    lexer::Lexer,
    parser::{Parser, diag_ctx::ParserDiagCtx},
};
use hycc_source::Source;
use hycc_util::ternary;

use crate::session::Session;

pub fn start(root_path: &str) {
    let mut session = Session::new(Source::new(root_path));
    compile(&mut session);

    let reporter = CLIReporter::new(&session.dctx, &session.source_registry);
    reporter.report();
}

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

pub fn compile(session: &mut Session) {
    let Some(tree) = analyze_source(session) else {
        return;
    };

    dbg!(tree);
}
