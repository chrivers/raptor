use std::fmt::{self, Debug, Display};

use crate::print::Theme;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct InstFrom {
    pub from: String,
}

impl Display for InstFrom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.keyword("FROM")?;
        f.src(&self.from)
    }
}

impl Debug for InstFrom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FROM {}", self.from)
    }
}
