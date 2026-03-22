use hycc_diagnostic::reporter::{CLIReporter, DiagnosticReporter};
use hycc_parser::lexer::Lexer;
use hycc_source::Source;

use crate::session::Session;

pub fn start(root_path: &str) {
    let mut session = Session::new(Source::new(root_path));
    compile(&mut session);
}

pub fn compile(session: &mut Session) {
    let mut lexer = Lexer::new(session.source_registry.root(), &mut session.dctx);
    lexer.tokenize();

    let reporter = CLIReporter::new(&session.dctx, &session.source_registry);
    reporter.report();
}
