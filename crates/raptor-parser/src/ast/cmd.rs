use std::fmt::{Debug, Display};

use camino::Utf8Path;

use crate::print::Theme;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InstCmd {
    pub cmd: Vec<String>,
}

impl Display for InstCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.keyword("CMD")?;
        f.dest(Utf8Path::new(&self.cmd[0]))?;
        for arg in &self.cmd[1..] {
            f.src(Utf8Path::new(arg.as_str()))?;
        }
        Ok(())
    }
}
