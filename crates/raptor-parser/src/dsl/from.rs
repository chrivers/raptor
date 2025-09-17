use std::fmt::{self, Debug, Display};

use crate::print::Theme;
use crate::util::module_name::ModuleName;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum FromSource {
    Raptor(ModuleName),
    Docker(String),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InstFrom {
    pub from: FromSource,
}

impl Display for InstFrom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.keyword("FROM")?;
        f.from(&self.from)
    }
}

impl Display for FromSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Raptor(src) => write!(f, "{src}"),
            Self::Docker(src) => write!(f, "docker://{src}"),
        }
    }
}
