use std::fmt::{Debug, Display};

use crate::dsl::{Chown, IncludeArg};
use crate::print::Theme;

#[derive(Clone, PartialEq, Eq)]
pub struct InstRender {
    pub src: String,
    pub dest: String,
    pub chmod: Option<u32>,
    pub chown: Option<Chown>,
    pub args: Vec<IncludeArg>,
}

impl Display for InstRender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl Debug for InstRender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RENDER ")?;
        if let Some(chmod) = &self.chmod {
            write!(f, "--chmod {chmod:04o} ")?;
        }
        if let Some(chown) = &self.chown {
            write!(f, "--chown {chown} ")?;
        }
        write!(f, "{:?} {:?}", self.src, self.dest)?;
        for arg in &self.args {
            write!(f, " {arg}")?;
        }
        Ok(())
    }
}
