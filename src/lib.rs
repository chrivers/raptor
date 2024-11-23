#[macro_use]
extern crate log;

pub mod client;
pub mod dsl;
pub mod parser;
pub mod program;
pub mod sandbox;
pub mod template;
pub mod util;

use crate::{dsl::Origin, parser::Rule};

#[derive(thiserror::Error, Debug)]
pub enum RaptorError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    MinijinjaError(#[from] minijinja::Error),

    #[error(transparent)]
    PestError(Box<pest::error::Error<Rule>>),

    #[error(transparent)]
    BincodeError(#[from] bincode::Error),

    #[error(transparent)]
    VarError(#[from] std::env::VarError),

    #[error(transparent)]
    Errno(#[from] nix::Error),

    #[error("Path is not valid utf-8: {0}")]
    BadPath(std::path::PathBuf),

    #[error("Script error: {0} {1:?}")]
    ScriptError(String, Origin),

    #[error("Undefined variable: {0}")]
    UndefinedVarError(String, Origin),

    #[error("RUN failed: {0}")]
    RunError(String),
}

impl From<pest_consume::Error<Rule>> for RaptorError {
    fn from(e: pest_consume::Error<Rule>) -> Self {
        Self::PestError(Box::new(e))
    }
}

impl From<std::path::PathBuf> for RaptorError {
    fn from(e: std::path::PathBuf) -> Self {
        Self::BadPath(e)
    }
}

pub type RaptorResult<T> = Result<T, RaptorError>;
