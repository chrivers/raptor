use std::fmt::{Debug, Display};

use camino::Utf8Path;

use crate::print::Theme;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InstEntrypoint {
    pub entrypoint: Vec<String>,
}

impl Display for InstEntrypoint {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.keyword("ENTRYPOINT")?;
        f.dest(Utf8Path::new(&self.entrypoint[0]))?;
        for arg in &self.entrypoint[1..] {
            f.src(Utf8Path::new(arg.as_str()))?;
        }
        Ok(())
    }
}
