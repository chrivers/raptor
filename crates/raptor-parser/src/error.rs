use camino::Utf8PathBuf;

use crate::Rule;

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error(transparent)]
    PestError(Box<pest::error::Error<Rule>>),

    #[error("Cannot get parent path from {0:?}")]
    BadPathNoParent(Utf8PathBuf),
}

impl From<pest_consume::Error<Rule>> for ParseError {
    fn from(e: pest_consume::Error<Rule>) -> Self {
        Self::PestError(Box::new(e))
    }
}
