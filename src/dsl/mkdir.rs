use std::fmt::{Debug, Display};

use camino::Utf8PathBuf;
use colored::Colorize;

use crate::dsl::Chown;
use crate::print::Theme;

#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub struct InstMkdir {
    pub dest: Utf8PathBuf,
    pub chmod: Option<u32>,
    pub chown: Option<Chown>,
    pub parents: bool,
}

impl Display for InstMkdir {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.keyword("MKDIR")?;
        if self.parents {
            write!(f, "{}", " -p".bright_white())?;
        }
        f.chmod(&self.chmod)?;
        f.chown(&self.chown)?;
        f.dest(&self.dest)?;
        Ok(())
    }
}
