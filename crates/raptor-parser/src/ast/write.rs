use std::fmt::{Debug, Display};

use camino::Utf8PathBuf;

use crate::ast::Chown;
use crate::print::Theme;

#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub struct InstWrite {
    pub dest: Utf8PathBuf,
    pub body: String,
    pub chmod: Option<u32>,
    pub chown: Option<Chown>,
}

impl Display for InstWrite {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.keyword("WRITE")?;
        f.chmod(&self.chmod)?;
        f.chown(&self.chown)?;
        f.value(&self.body)?;
        f.dest(&self.dest)?;
        Ok(())
    }
}
