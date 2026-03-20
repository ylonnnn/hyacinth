#[cfg(test)]
mod tests {
    use hyacinth::prelude::*;
    use program::Source;

    #[test]
    fn lexer_from_arbitrary_string() {
        let source = r"1 + 2";
        let mut lexer = Lexer::new(Source::new(file!().into(), source.into()));

        lexer.tokenize();

        assert_eq!(lexer.len(), 4);
    }

    #[test]
    fn lexer_from_file() {
        let source = "hyc/lexer/first.hyc";
        let mut lexer = Lexer::new(Source::new_from_file(source.into()));

        lexer.tokenize();

        assert!(lexer.len() > 0); // Merely expects the program to detect and lex its contents
    }

    #[test]
    fn lexer_ignore_comments() {
        let source = r#"
            // hello world
            // this is a comment
            1 + 2
            // the only recognized tokens are
            // "1", "+", "2", and multiple line feeds
        "#;
        let mut lexer = Lexer::new(Source::new(file!().into(), source.into()));

        lexer.tokenize();

        assert_eq!(lexer.len(), 10);
    }

    #[test]
    fn lexer_delimeters() {
        let source = r"[]{}().,;:";
        let mut lexer = Lexer::new(Source::new(file!().into(), source.into()));

        lexer.tokenize();

        assert_eq!(lexer.len(), source.len() + 1);
    }

    #[test]
    fn lexer_valid_int() {
        let sources = vec!["123", "0b0101", "0o7123", "0xabf3d2"];

        sources.iter().for_each(|source| {
            let mut lexer = Lexer::new(Source::new(file!().into(), (*source).into()));
            lexer.tokenize();

            assert_eq!(lexer.len(), 2);
        });
    }

    #[test]
    fn lexer_invalid_int() {
        let sources = vec!["9m2", "0b1923", "0o123854", "0xhbh2331fe"];

        sources.iter().for_each(|source| {
            let mut lexer = Lexer::new(Source::new(file!().into(), (*source).into()));
            lexer.tokenize();

            assert_ne!(lexer.diagnostics.data().len(), 0);
        });
    }

    #[test]
    fn lexer_valid_chars() {
        let sources = vec!["'2'", "'\\t'", "'\\''", "'v'"];

        sources.iter().for_each(|source| {
            let mut lexer = Lexer::new(Source::new(file!().into(), (*source).into()));
            lexer.tokenize();

            lexer.diagnostics.data().iter().for_each(|d| {
                dbg!(&d.message);
            });

            assert_eq!(lexer.diagnostics.data().len(), 0);
        });
    }

    #[test]
    fn lexer_invaid_chars() {
        let sources = vec!["'220'", "'unterminated char seq"];

        sources.iter().for_each(|source| {
            let mut lexer = Lexer::new(Source::new(file!().into(), (*source).into()));
            lexer.tokenize();

            assert_ne!(lexer.diagnostics.data().len(), 0);
        });
    }

    #[test]
    fn lexer_valid_str() {
        let sources = vec!["\"hello world\"", "\"hello \\\"valid\\\" world\""];

        sources.iter().for_each(|source| {
            let mut lexer = Lexer::new(Source::new(file!().into(), (*source).into()));
            lexer.tokenize();

            lexer.diagnostics.data().iter().for_each(|d| {
                dbg!(&d.message);
            });

            assert_eq!(lexer.diagnostics.data().len(), 0);
        });
    }

    #[test]
    fn lexer_invaid_str() {
        let sources = vec!["\"unterminated char seq string", "\"hello \"world\\\""];

        sources.iter().for_each(|source| {
            let mut lexer = Lexer::new(Source::new(file!().into(), (*source).into()));
            lexer.tokenize();

            assert_ne!(lexer.diagnostics.data().len(), 0);
        });
    }
}
