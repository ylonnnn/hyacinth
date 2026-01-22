use crate::{
    core::diagnostic::{Diagnostic, DiagnosticSeverity, code::DiagnosticErrorKind},
    syntax::{Token, TokenKind},
    ternary,
};

pub fn syntax_error_unexpected_token(source: &String, token: &Token) -> Diagnostic {
    Diagnostic::new(
        token.span.clone(),
        DiagnosticSeverity::Error,
        DiagnosticErrorKind::UnexpectedToken.into(),
        format!(
            "unexpected token `{}`.",
            token.view(source).replace("\n", "\\n"),
        ),
    )
}

pub fn syntax_error_expectation_mismatch(
    source: &String,
    token: &Token,
    expected: Option<TokenKind>,
) -> Diagnostic {
    Diagnostic::new(
        token.span.clone(),
        DiagnosticSeverity::Error,
        DiagnosticErrorKind::UnexpectedToken.into(),
        format!(
            "unexpected token `{}`{}",
            token.view(source).replace("\n", "\\n"),
            ternary!(
                expected.is_some(),
                format!(", expected `{}`", expected.unwrap().to_string()),
                ".".into()
            ),
        ),
    )
}
