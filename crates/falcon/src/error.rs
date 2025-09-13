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
}

pub type FalconResult<T> = Result<T, FalconError>;
