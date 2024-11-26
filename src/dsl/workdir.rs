use std::fmt::{Debug, Display};

use crate::print::Theme;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InstWorkdir {
    pub dir: String,
}

impl Display for InstWorkdir {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.keyword("WORKDIR")?;
        f.dest(&self.dir)
    }
}
