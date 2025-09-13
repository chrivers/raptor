#[macro_use]
extern crate log;

pub mod build;
pub mod dsl;
pub mod parser;
pub mod print;
pub mod program;
pub mod sandbox;
pub mod template;
pub mod util;

use std::os::unix::net::UnixStream;
use std::process::ExitStatus;
use std::sync::mpsc;

use camino::Utf8PathBuf;

use crate::dsl::Origin;
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
    BincodeDecodeError(#[from] bincode::error::DecodeError),

    #[error(transparent)]
    BincodeEncodeError(#[from] bincode::error::EncodeError),

    #[error(transparent)]
    VarError(#[from] std::env::VarError),

    #[error(transparent)]
    Errno(#[from] nix::Error),

    #[error(transparent)]
    MpscTimeout(#[from] mpsc::RecvTimeoutError),

    #[error(transparent)]
    SendError(#[from] mpsc::SendError<UnixStream>),

    #[error(transparent)]
    FmtError(#[from] std::fmt::Error),

    #[error(transparent)]
    DockerError(#[from] dregistry::error::DockerError),

    #[error(transparent)]
    FalconError(#[from] falcon::error::FalconError),

    #[error("Error while checking cache status of {0:?}: {1}")]
    CacheIoError(Utf8PathBuf, std::io::Error),

    #[error("Cannot get parent path from {0:?}")]
    BadPathNoParent(Utf8PathBuf),

    #[error("Script error: {0}")]
    ScriptError(String, Origin),

    #[error("Undefined variable: {0}")]
    UndefinedVarError(String, Origin),

    #[error("Sandbox error: {0}")]
    SandboxRequestError(nix::errno::Errno),

    #[error("process exit status {0}")]
    SandboxRunError(ExitStatus),
}

impl From<pest_consume::Error<Rule>> for RaptorError {
    fn from(e: pest_consume::Error<Rule>) -> Self {
        Self::PestError(Box::new(e))
    }
}

impl RaptorError {
    #[must_use]
    pub const fn category(&self) -> &'static str {
        match self {
            Self::IoError(_) => "IO Error",
            Self::MinijinjaError(_) => "Template error",
            Self::PestError(_) => "Parser error",
            Self::BincodeEncodeError(_) => "Encoding error",
            Self::BincodeDecodeError(_) => "Decoding error",
            Self::VarError(_) => "Environment error",
            Self::Errno(_) => "Errno",
            Self::CacheIoError(_, _) => "Cache io error",
            Self::BadPathNoParent(_) => "Path error",
            Self::ScriptError(_, _) => "Script error",
            Self::UndefinedVarError(_, _) => "Undefined variable",
            Self::SandboxRequestError(_) => "Sandbox request error",
            Self::SandboxRunError(_) => "Sandbox run error",
            Self::MpscTimeout(_) => "Channel error",
            Self::SendError(_) => "Send error",
            Self::FmtError(_) => "Format error",
            Self::DockerError(_) => "Docker error",
            Self::FalconError(_) => "Falcon error",
        }
    }
}

pub type RaptorResult<T> = Result<T, RaptorError>;
