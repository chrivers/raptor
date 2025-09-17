use camino::Utf8PathBuf;

use crate::Rule;
use crate::dsl::Origin;

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error(transparent)]
    MinijinjaError(#[from] minijinja::Error),

    #[error(transparent)]
    PestError(Box<pest::error::Error<Rule>>),

    #[error("Cannot get parent path from {0:?}")]
    BadPathNoParent(Utf8PathBuf),

    #[error("Undefined variable: {0}")]
    UndefinedVarError(String, Origin),
}

impl From<pest_consume::Error<Rule>> for ParseError {
    fn from(e: pest_consume::Error<Rule>) -> Self {
        Self::PestError(Box::new(e))
    }
}
