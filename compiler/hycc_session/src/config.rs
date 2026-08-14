use hycc_ast::token::TokenKind;

#[macro_export]
macro_rules! hyc_file_ext {
    () => {
        "hyc"
    };
}

pub const HYC_FILE_EXT: &str = hyc_file_ext!();
pub const HYC_DIR_PETAL_FILE: &str = concat!("petal", ".", hyc_file_ext!());

pub const HYC_PATH_SEP_TOK_KIND: TokenKind = TokenKind::ColonColon;

pub const HYC_STRUCT_FIELD_LIMIT: usize = 64;
