#[macro_use]
extern crate log;

pub mod build;
pub mod dsl;
pub mod program;
pub mod runner;
pub mod sandbox;
pub mod template;
pub mod util;

use std::os::unix::net::UnixStream;
use std::process::ExitStatus;
use std::sync::mpsc;

use camino::Utf8PathBuf;

use raptor_parser::ast::{InstMount, MountType, Origin};
use raptor_parser::util::Location;

#[derive(thiserror::Error, Debug)]
pub enum RaptorError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    MinijinjaError(#[from] minijinja::Error),

    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),

    #[error(transparent)]
    VarError(#[from] std::env::VarError),

    #[error(transparent)]
    Errno(#[from] nix::Error),

    #[error(transparent)]
    MpscTimeout(#[from] mpsc::RecvTimeoutError),

    #[error(transparent)]
    SendError(#[from] mpsc::SendError<UnixStream>),

    #[error(transparent)]
    DockerError(#[from] dregistry::error::DockerError),

    #[error(transparent)]
    FalconError(#[from] falcon::error::FalconError),

    #[error(transparent)]
    SafeParentError(#[from] raptor_parser::util::SafeParentError),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error(transparent)]
    ParseTomlError(#[from] toml::de::Error),

    #[error("Parse error: {0:?}")]
    ParseError(Location<raptor_parser::ParseError>),

    #[error("Undefined variable: {0}")]
    UndefinedVarError(String, Origin),

    #[error("Error while checking cache status of {0:?}: {1}")]
    CacheIoError(Utf8PathBuf, std::io::Error),

    #[error("Script error: {0}")]
    ScriptError(String, Origin),

    #[error("Sandbox error: {0}")]
    SandboxRequestError(nix::errno::Errno),

    #[error("process exit status {0}")]
    SandboxRunError(ExitStatus),

    #[error("Required mount [{}] not specified", .0.name)]
    MountMissing(InstMount),

    #[error("Raptor requires root to run (please try again with sudo)")]
    RootRequired,

    #[error("Only a single source is supported for mounts of type {0:?}")]
    SingleMountOnly(MountType),

    #[error("Invalid layer cache name")]
    LayerCacheParseError,

    #[error("Package not found: ${0}")]
    PackageNotFound(String, Origin),
}

impl RaptorError {
    #[must_use]
    pub const fn category(&self) -> &'static str {
        match self {
            Self::IoError(_) => "IO Error",
            Self::MinijinjaError(_) => "Template error",
            Self::SerdeJsonError(_) => "Serde json error",
            Self::ParseError(_) => "Parser error",
            Self::VarError(_) => "Environment error",
            Self::Errno(_) => "Errno",
            Self::CacheIoError(_, _) => "Cache io error",
            Self::ScriptError(_, _) => "Script error",
            Self::SandboxRequestError(_) => "Sandbox request error",
            Self::SandboxRunError(_) => "Sandbox run error",
            Self::MpscTimeout(_) => "Channel error",
            Self::SendError(_) => "Send error",
            Self::DockerError(_) => "Docker error",
            Self::FalconError(_) => "Falcon error",
            Self::MountMissing(_) => "Missing mount error",
            Self::RootRequired => "Root required",
            Self::SafeParentError(_) => "Safe parent error",
            Self::UndefinedVarError(_, _) => "Undefined var error",
            Self::SingleMountOnly(_) => "Single mount error",
            Self::ParseIntError(_) => "Parse int error",
            Self::ParseTomlError(_) => "Parse toml error",
            Self::LayerCacheParseError => "Layer cache parse error",
            Self::PackageNotFound(_, _) => "Package not found",
        }
    }
}

impl From<Location<raptor_parser::ParseError>> for RaptorError {
    fn from(value: Location<raptor_parser::ParseError>) -> Self {
        Self::ParseError(value)
    }
}

pub type RaptorResult<T> = Result<T, RaptorError>;
