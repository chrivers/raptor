use std::fmt::Display;

use crate::dsl::Chown;
use crate::print::Theme;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InstCopy {
    pub srcs: Vec<String>,
    pub dest: String,
    pub chmod: Option<u32>,
    pub chown: Option<Chown>,
}

impl Display for InstCopy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.keyword("COPY")?;
        f.chmod(&self.chmod)?;
        f.chown(&self.chown)?;
        for src in &self.srcs {
            f.src(src)?;
        }
        f.dest(&self.dest)
    }
}
