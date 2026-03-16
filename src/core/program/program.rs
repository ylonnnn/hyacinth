use crate::{
    core::{
        diagnostic::{
            DiagnosticList,
            reporter::{cli_reporter::CLIReporter, reporter::DiagnosticReporter},
        },
        source::ProgramSource,
    },
    syntax::lexer::Lexer,
};

#[derive(Debug)]
pub struct Program {
    pub source: ProgramSource,
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
        let source = ProgramSource::new_from_file(path);
        Self {
            lexer: Lexer::new(source.clone()),
            source,

            state: ProgramState::None,
            diagnostics: DiagnosticList::default(),
        }
    }

    pub fn new_with_state(path: &str, state: ProgramState) -> Self {
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

        // let mut parser = Parser::new(self);
        // if let Some(_) = parser.parse() {
        //     // dbg!(&node);
        // }
    }

    pub fn compile(&mut self) {
        self.analyze();

        let reporter = CLIReporter::new(self);
        reporter.report();
    }
}
