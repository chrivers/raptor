use camino::{Utf8Path, Utf8PathBuf};

#[derive(thiserror::Error, Debug)]
pub enum SafeParentError {
    #[error("Cannot get parent path from {0:?}")]
    BadPathNoParent(Utf8PathBuf),
}

pub trait SafeParent {
    fn try_parent(&self) -> Result<&Utf8Path, SafeParentError>;
}

impl SafeParent for Utf8Path {
    fn try_parent(&self) -> Result<&Utf8Path, SafeParentError> {
        self.parent()
            .ok_or_else(|| SafeParentError::BadPathNoParent(self.into()))
    }
}

impl SafeParent for Utf8PathBuf {
    fn try_parent(&self) -> Result<&Utf8Path, SafeParentError> {
        self.parent()
            .ok_or_else(|| SafeParentError::BadPathNoParent(self.into()))
    }
}
