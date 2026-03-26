use hycc_diagnostic::{Diagnostic, DiagnosticContext};
use hycc_util::ternary;

#[derive(Debug)]
pub struct ParserDiagCtx<'d> {
    data: &'d mut Vec<Diagnostic>,
    state: ParserDiagCtxState,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum ParserDiagCtxState {
    Synchronized,
    Disarray,
}

impl<'d> ParserDiagCtx<'d> {
    pub fn new(data: &'d mut Vec<Diagnostic>) -> Self {
        Self {
            data,
            state: ParserDiagCtxState::Synchronized,
        }
    }

    pub fn sync(&mut self) {
        self.state = ParserDiagCtxState::Synchronized
    }

    pub fn is(&self, state: ParserDiagCtxState) -> bool {
        self.state == state
    }

    pub fn is_in_disarray(&self) -> bool {
        self.is(ParserDiagCtxState::Disarray)
    }
}

impl<'d> DiagnosticContext for ParserDiagCtx<'d> {
    fn data(&self) -> &Vec<Diagnostic> {
        &self.data
    }

    fn data_mut(&mut self) -> &mut Vec<Diagnostic> {
        &mut self.data
    }

    fn add(&mut self, diagnostic: Diagnostic) -> Option<&mut Diagnostic> {
        let is_err = diagnostic.is_error();
        if is_err {
            if self.is(ParserDiagCtxState::Disarray) {
                return None;
            }

            self.state = ParserDiagCtxState::Disarray;
        }

        let data = self.data_mut();

        data.push(diagnostic);
        data.last_mut()
    }

    fn error(
        &mut self,
        code: hycc_diagnostic::code::DiagnosticCode,
        message: &str,
        span: hycc_span::Span,
    ) -> Option<&Diagnostic> {
        ternary!(self.is(ParserDiagCtxState::Disarray), None, {
            DiagnosticContext::error(self, code, message, span)
        })
    }
}
