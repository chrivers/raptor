use std::fmt::Debug;
use std::ops::Range;
use std::sync::Arc;

use camino::{Utf8Path, Utf8PathBuf};

use crate::util::SafeParent;
use crate::{ParseError, ParseErrorDetails};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Origin {
    pub path: Arc<Utf8PathBuf>,
    pub span: Range<usize>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Location<T> {
    pub origin: Origin,
    pub inner: T,
}

impl<T> Location<T> {
    pub fn origin(&self) -> Origin {
        self.origin.clone()
    }

    pub const fn make(origin: Origin, inner: T) -> Self {
        Self { origin, inner }
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

    pub fn basedir(&self) -> Result<&Utf8Path, ParseError> {
        self.path.try_parent().map_err(|err| ParseError {
            path: self.path.clone(),
            details: ParseErrorDetails::PathParse(err),
        })
    }
}
