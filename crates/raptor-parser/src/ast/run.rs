use std::fmt::{Debug, Display};

use camino::Utf8Path;

use crate::print::Theme;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InstRun {
    pub run: Vec<String>,
}

impl Display for InstRun {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.keyword("RUN")?;
        f.dest(Utf8Path::new(&self.run[0]))?;
        for arg in &self.run[1..] {
            f.src(Utf8Path::new(arg.as_str()))?;
        }
        Ok(())
    }
}
