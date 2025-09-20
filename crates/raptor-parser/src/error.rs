use camino::Utf8PathBuf;

use lalrpop_util::ParseError as LalrParseError;
use lalrpop_util::lexer::Token;

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Cannot get parent path from {0:?}")]
    BadPathNoParent(Utf8PathBuf),

    #[error("parse error")]
    ParseError(String),
}

impl<'a> From<LalrParseError<usize, Token<'a>, &'a str>> for ParseError {
    fn from(value: LalrParseError<usize, Token<'a>, &'a str>) -> Self {
        Self::ParseError(value.to_string())
    }
}
