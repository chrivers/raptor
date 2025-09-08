#[derive(thiserror::Error, Debug)]
pub enum DockerError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    FromHexError(#[from] hex::FromHexError),

    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Could not parse digest")]
    DigestError,

    #[error("Manifest not found for selected os/architecture")]
    ManifestNotFound,
}

pub type DResult<T> = Result<T, DockerError>;
