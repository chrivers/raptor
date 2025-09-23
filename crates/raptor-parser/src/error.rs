use camino::Utf8PathBuf;

use crate::Rule;
use crate::lexer::LexerError;

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error(transparent)]
    PestError(Box<pest::error::Error<Rule>>),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    LexerError(#[from] LexerError),

    #[error("Unexpected eof")]
    UnexpectedEof,

    #[error("Expected word")]
    ExpectedWord,

    #[error("Cannot get parent path from {0:?}")]
    BadPathNoParent(Utf8PathBuf),
}

impl From<pest_consume::Error<Rule>> for ParseError {
    fn from(e: pest_consume::Error<Rule>) -> Self {
        Self::PestError(Box::new(e))
    }
}
