use std::fmt::Debug;

use crate::dsl::Chown;

pub struct InstWrite {
    pub dest: String,
    pub body: String,
    pub chmod: Option<u16>,
    pub chown: Option<Chown>,
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
        write!(f, "{} {:?}", self.dest, self.body)?;
        Ok(())
    }
}
