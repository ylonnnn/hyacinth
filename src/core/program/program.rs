use crate::{
    core::diagnostic::{
        DiagnosticList,
        reporter::{cli_reporter::CLIReporter, reporter::DiagnosticReporter},
    },
    syntax::{Lexer, LexerSourceOrigin, parser::parser::Parser},
};

#[derive(Debug)]
pub struct Program {
    pub path: String,
    pub lexer: Lexer,

    state: ProgramState,
    diagnostics: DiagnosticList,
}

#[repr(u32)]
#[derive(Debug)]
pub enum ProgramState {
    None,
    Analyzed,
}

impl Program {
    pub fn new(path: &str) -> Self {
        Self {
            lexer: Lexer::new(LexerSourceOrigin::File(path.to_owned())),
            path: path.to_owned(),

            state: ProgramState::None,
            diagnostics: DiagnosticList::default(),
        }
    }

    pub fn with(path: &str, state: ProgramState) -> Self {
        let mut inst = Program::new(path);
        inst.state = state;

        inst
    }

    pub fn diagnostic_list(&self) -> &DiagnosticList {
        &self.diagnostics
    }

    pub fn diagnostic_list_mut(&mut self) -> &mut DiagnosticList {
        &mut self.diagnostics
    }

    pub fn analyze(&mut self) {
        self.lexer.tokenize();
        std::mem::swap(&mut self.diagnostics, &mut self.lexer.diagnostics);

        let mut parser = Parser::new(self);
        if let Some(node) = parser.parse() {
            dbg!(&node);
        }
    }

    pub fn compile(&mut self) {
        self.analyze();

        let reporter = CLIReporter::new(self);
        reporter.report();
    }
}
