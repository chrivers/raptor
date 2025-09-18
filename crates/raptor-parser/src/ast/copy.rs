use std::fmt::Display;

use camino::Utf8PathBuf;

use crate::ast::Chown;
use crate::print::Theme;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InstCopy {
    pub srcs: Vec<Utf8PathBuf>,
    pub dest: Utf8PathBuf,
    pub chmod: Option<u32>,
    pub chown: Option<Chown>,
}

impl Display for InstCopy {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.keyword("COPY")?;
        f.chmod(&self.chmod)?;
        f.chown(&self.chown)?;
        for src in &self.srcs {
            f.src(src)?;
        }
        f.dest(&self.dest)
    }
}
