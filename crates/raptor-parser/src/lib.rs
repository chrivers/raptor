pub mod ast;
pub mod error;
pub mod print;
pub mod unescape;
pub mod util;

use std::sync::Arc;

use camino::Utf8Path;

use crate::ast::Statement;

pub use error::{ParseError, ParseErrorDetails};
pub type ParseResult<T> = Result<T, ParseErrorDetails>;

pub fn parse(filename: &str, data: &str) -> Result<Vec<Statement>, ParseError> {
    let path = Arc::new(Utf8Path::new(filename).to_path_buf());
    parser::FileParser::new()
        .parse(&path, data)
        .map_err(|err| ParseError {
            path,
            details: err.into(),
        })
}

lalrpop_util::lalrpop_mod!(
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::unnested_or_patterns)]
    #[allow(clippy::redundant_pub_crate)]
    #[allow(clippy::elidable_lifetime_names)]
    #[allow(clippy::missing_const_for_fn)]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::match_same_arms)]
    #[allow(clippy::must_use_candidate)]
    #[allow(clippy::option_if_let_else)]
    #[allow(clippy::no_effect_underscore_binding)]
    #[allow(clippy::cloned_instead_of_copied)]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    #[allow(clippy::needless_raw_string_hashes)]
    #[allow(clippy::unnecessary_wraps)]
    #[allow(clippy::use_self)]
    #[allow(clippy::cognitive_complexity)]
    #[allow(clippy::useless_conversion)]
    #[allow(unused_lifetimes)]
    #[allow(unused_qualifications)]
    #[rustfmt::skip]
    pub parser
);
