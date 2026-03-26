pub(crate) mod errors {
    use hycc_ast::token::{Token, TokenKind};
    use hycc_diagnostic::{Diagnostic, DiagnosticSeverity, code::DiagnosticErrorKind};
    use hycc_source::Source;
    use hycc_util::ternary;

    pub fn unexpected_token(source: &Source, token: &Token, expected: Option<&str>) -> Diagnostic {
        Diagnostic::new(
            token.span.clone(),
            DiagnosticSeverity::Error,
            DiagnosticErrorKind::UnexpectedToken.into(),
            format!(
                "unexpected `{}`, {}.",
                token.view(&source.data).replace("\n", "\\n"),
                ternary!(expected.is_some(), expected.unwrap(), "")
            ),
        )
    }

    pub fn token_kind_mismatch(
        source: &Source,
        token: &Token,
        expected: Option<TokenKind>,
    ) -> Diagnostic {
        let expectation_str = format!("expected `{}`", expected.unwrap().to_string());
        unexpected_token(
            source,
            token,
            ternary!(expected.is_some(), Some(&expectation_str), None),
        )
    }
}
