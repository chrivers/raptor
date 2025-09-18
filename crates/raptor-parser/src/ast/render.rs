use std::fmt::{Debug, Display};

use camino::Utf8PathBuf;

use crate::ast::{Chown, IncludeArg};
use crate::print::Theme;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InstRender {
    pub src: Utf8PathBuf,
    pub dest: Utf8PathBuf,
    pub chmod: Option<u32>,
    pub chown: Option<Chown>,
    pub args: Vec<IncludeArg>,
}

impl Display for InstRender {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.keyword("RENDER")?;
        f.chmod(&self.chmod)?;
        f.chown(&self.chown)?;
        f.src(&self.src)?;
        f.dest(&self.dest)?;
        for arg in &self.args {
            f.include_arg(arg)?;
        }
        Ok(())
    }
}
