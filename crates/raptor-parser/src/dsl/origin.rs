use std::fmt::Debug;
use std::ops::Range;
use std::sync::Arc;

use camino::{Utf8Path, Utf8PathBuf};

use crate::util::SafeParent;
use crate::ParseResult;

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
    pub fn from_node(node: &crate::ast::Node) -> Self {
        let span = node.as_span();
        Self {
            path: node.user_data().path.clone(),
            span: span.start()..span.end(),
        }
    }

    pub fn basedir(&self) -> ParseResult<&Utf8Path> {
        self.path.try_parent()
    }
}
