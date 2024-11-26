use std::fmt::{self, Debug, Display};

use crate::print::Theme;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InstInvoke {
    pub args: Vec<String>,
}

impl Display for InstInvoke {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.keyword("INVOKE")?;
        for arg in &self.args {
            f.src(arg)?;
        }
        Ok(())
    }
}
