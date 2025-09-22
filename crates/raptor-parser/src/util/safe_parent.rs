use camino::{Utf8Path, Utf8PathBuf};

use crate::error::PathParseError;

pub trait SafeParent {
    fn try_parent(&self) -> Result<&Utf8Path, PathParseError>;
}

impl SafeParent for Utf8Path {
    fn try_parent(&self) -> Result<&Utf8Path, PathParseError> {
        self.parent()
            .ok_or_else(|| PathParseError::BadPathNoParent(self.into()))
    }
}

impl SafeParent for Utf8PathBuf {
    fn try_parent(&self) -> Result<&Utf8Path, PathParseError> {
        self.parent()
            .ok_or_else(|| PathParseError::BadPathNoParent(self.into()))
    }
}
