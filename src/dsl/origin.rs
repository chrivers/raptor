use std::fmt::Debug;
use std::ops::Range;
use std::sync::Arc;

use camino::{Utf8Path, Utf8PathBuf};

use crate::RaptorResult;

#[derive(Clone, Hash, PartialEq, Eq)]
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
    pub fn from_node(node: &crate::parser::ast::Node) -> Self {
        let span = node.as_span();
        Self {
            path: node.user_data().path.clone(),
            span: span.start()..span.end(),
        }
    }

    pub fn basedir(&self) -> RaptorResult<&Utf8Path> {
        self.path
            .parent()
            .ok_or_else(|| crate::RaptorError::BadPathNoParent(self.path.as_ref().clone()))
    }
}

impl Debug for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{:<15} {:>3} .. {:>3}]",
            self.path, self.span.start, self.span.end
        )
    }
}
