use std::fmt::{Debug, Display};

use crate::print::Theme;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InstRun {
    pub run: Vec<String>,
}

impl Display for InstRun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.keyword("RUN")?;
        f.dest(&self.run[0])?;
        for arg in &self.run[1..] {
            f.src(arg)?;
        }
        Ok(())
    }
}
