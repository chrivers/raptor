use crate::reference::Rule;

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

    #[error(transparent)]
    ToStrError(#[from] reqwest::header::ToStrError),

    #[error(transparent)]
    PestError(#[from] Box<pest_consume::Error<Rule>>),

    #[error(transparent)]
    ParseError(#[from] crate::authparse::ParseError),

    #[error("Could not parse digest")]
    DigestError,

    #[error("Manifest not found for selected os/architecture")]
    ManifestNotFound,
}

pub type DResult<T> = Result<T, DockerError>;

impl From<pest_consume::Error<Rule>> for DockerError {
    fn from(e: pest_consume::Error<Rule>) -> Self {
        Self::PestError(Box::new(e))
    }
}
