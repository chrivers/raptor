#[derive(thiserror::Error, Debug)]
pub enum FalconError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    BincodeDecodeError(#[from] bincode::error::DecodeError),

    #[error(transparent)]
    BincodeEncodeError(#[from] bincode::error::EncodeError),

    #[error(transparent)]
    Errno(#[from] nix::Error),

    #[error("Sandbox error: {0}")]
    SandboxRequestError(nix::errno::Errno),
}

pub type FalconResult<T> = Result<T, FalconError>;
