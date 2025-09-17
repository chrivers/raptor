use camino::{Utf8Path, Utf8PathBuf};

use crate::ParseResult;

pub trait SafeParent {
    fn try_parent(&self) -> ParseResult<&Utf8Path>;
}

impl SafeParent for Utf8Path {
    fn try_parent(&self) -> ParseResult<&Utf8Path> {
        self.parent()
            .ok_or_else(|| crate::ParseError::BadPathNoParent(self.into()))
    }
}

impl SafeParent for Utf8PathBuf {
    fn try_parent(&self) -> ParseResult<&Utf8Path> {
        self.parent()
            .ok_or_else(|| crate::ParseError::BadPathNoParent(self.into()))
    }
}
