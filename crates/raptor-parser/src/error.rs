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

    #[error(transparent)]
    ClapError(#[from] clap::error::Error),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error(
        "Invalid permission mask\n\nValue must specified as 3 or 4 octal digits (0755, 1777, 644, 640, etc)"
    )]
    InvalidPermissionMask,

    #[error("Unexpected eof")]
    UnexpectedEof,

    #[error("Expected end of line")]
    ExpectedEol,

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
