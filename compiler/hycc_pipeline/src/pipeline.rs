use hycc_diagnostic::{
    DiagnosticContext,
    reporter::{CLIReporter, DiagnosticReporter},
};
use hycc_parser::{
    lexer::Lexer,
    parser::{Parser, diag_ctx::ParserDiagCtx},
};
use hycc_source::Source;

use crate::session::Session;

pub fn start(root_path: &str) {
    let mut session = Session::new(Source::new(root_path));
    compile(&mut session);
}

pub fn compile(session: &mut Session) {
    let mut lexer = Lexer::new(session.source_registry.root(), &mut session.dctx);
    let tok_stream = lexer.tokenize();

    let mut parser = Parser::new(
        &session.source_registry.root(),
        ParserDiagCtx::new(session.dctx.data_mut()),
        tok_stream,
    );
    parser.parse();

    let reporter = CLIReporter::new(&session.dctx, &session.source_registry);
    reporter.report();
}
