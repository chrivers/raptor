use std::fmt::{Debug, Display};

use crate::dsl::{Chown, IncludeArg};
use crate::print::Theme;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InstRender {
    pub src: String,
    pub dest: String,
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
