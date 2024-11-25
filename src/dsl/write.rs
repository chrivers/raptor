use std::fmt::{Debug, Display};

use crate::dsl::Chown;
use crate::print::Theme;

#[derive(Clone, PartialEq, Eq)]
pub struct InstWrite {
    pub dest: String,
    pub body: String,
    pub chmod: Option<u32>,
    pub chown: Option<Chown>,
}

impl Display for InstWrite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.keyword("WRITE")?;
        f.chmod(&self.chmod)?;
        f.chown(&self.chown)?;
        f.dest(&self.dest)?;
        f.value(&self.body)?;
        Ok(())
    }
}

impl Debug for InstWrite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WRITE ")?;
        if let Some(chmod) = &self.chmod {
            write!(f, "--chmod {chmod:04o} ")?;
        }
        if let Some(chown) = &self.chown {
            write!(f, "--chown {chown} ")?;
        }
        write!(f, "{:?} {:?}", self.dest, self.body)?;
        Ok(())
    }
}
