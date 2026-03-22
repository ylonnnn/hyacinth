use hycc_parser::lexer::Lexer;
use hycc_source::Source;

use crate::session::Session;

pub fn start(root_path: &str) {
    let mut session = Session::new(Source::new(root_path));
    compile(&mut session);
}

pub fn compile(session: &mut Session) {
    let mut lexer = Lexer::new(&session.source_tree.root.data);
    lexer.tokenize();
}
