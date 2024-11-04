use std::fmt::Debug;

use crate::dsl::Chown;

pub struct InstCopy {
    pub srcs: Vec<String>,
    pub dest: String,
    pub chmod: Option<u16>,
    pub chown: Option<Chown>,
}

impl Debug for InstCopy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "COPY ")?;
        if let Some(chmod) = &self.chmod {
            write!(f, "--chmod {chmod:04o} ")?;
        }
        if let Some(chown) = &self.chown {
            write!(f, "--chown {chown} ")?;
        }
        for src in &self.srcs {
            write!(f, "{src:?} ")?;
        }
        write!(f, "{:?}", self.dest)
    }
}
