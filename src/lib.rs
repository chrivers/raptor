#![warn(
    clippy::all,
    clippy::correctness,
    clippy::pedantic,
    clippy::cargo,
    clippy::nursery,
    clippy::perf,
    clippy::style
)]
#![allow(
    clippy::multiple_crate_versions,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::module_inception,
    clippy::similar_names
)]

#[macro_use]
extern crate log;

pub mod client;
pub mod dsl;
pub mod parser;
pub mod sandbox;
pub mod template;
pub mod util;

use crate::parser::Rule;

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
}

impl From<pest_consume::Error<Rule>> for RaptorError {
    fn from(e: pest_consume::Error<Rule>) -> Self {
        Self::PestError(Box::new(e))
    }
}

pub type RaptorResult<T> = std::result::Result<T, RaptorError>;
