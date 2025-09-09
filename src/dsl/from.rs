use std::fmt::{self, Debug, Display};

use crate::print::Theme;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum FromSource {
    Plain(String),
    Docker(String),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InstFrom {
    pub from: String,
}

impl Display for InstFrom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.keyword("FROM")?;
        f.src(&self.from)
    }
}
