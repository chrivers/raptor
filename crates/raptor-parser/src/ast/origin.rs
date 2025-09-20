use std::fmt::Debug;
use std::ops::Range;
use std::sync::Arc;

use camino::{Utf8Path, Utf8PathBuf};

use crate::ParseResult;
use crate::util::SafeParent;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Origin {
    pub path: Arc<Utf8PathBuf>,
    pub span: Range<usize>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Location<T> {
    pub path: Arc<Utf8PathBuf>,
    pub span: Range<usize>,
    pub inner: T,
}

impl<T> Location<T> {
    pub fn origin(&self) -> Origin {
        Origin::make(&*self.path, self.span.clone())
    }
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
    // FIXME: remove after lalrpop rework
    pub fn blank() -> Self {
        Self {
            path: Arc::new(Utf8PathBuf::new()),
            span: 0..0,
        }
    }

    pub fn basedir(&self) -> ParseResult<&Utf8Path> {
        self.path.try_parent()
    }
}
