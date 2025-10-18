use std::fmt::Debug;
use std::ops::Range;
use std::sync::Arc;

use camino::{Utf8Path, Utf8PathBuf};

use crate::util::{SafeParent, SafeParentError};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Origin {
    pub path: Arc<Utf8PathBuf>,
    pub span: Range<usize>,
}

impl Origin {
    #[must_use]
    pub const fn new(path: Arc<Utf8PathBuf>, span: Range<usize>) -> Self {
        Self { path, span }
    }

    pub fn make(path: impl AsRef<Utf8Path>, span: Range<usize>) -> Self {
        Self::new(Arc::new(path.as_ref().into()), span)
    }

    #[must_use]
    pub fn inline() -> Self {
        Self::make("<inline>", 0..0)
    }

    pub fn basedir(&self) -> Result<&Utf8Path, SafeParentError> {
        self.path.try_parent()
    }
}
