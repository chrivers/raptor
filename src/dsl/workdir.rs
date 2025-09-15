use std::fmt::{Debug, Display};

use camino::Utf8PathBuf;

use crate::print::Theme;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InstWorkdir {
    pub dir: Utf8PathBuf,
}

impl Display for InstWorkdir {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.keyword("WORKDIR")?;
        f.dest(&self.dir)
    }
}
