use std::fmt::Debug;

use crate::dsl::Chown;

#[derive(Clone)]
pub struct InstRender {
    pub src: String,
    pub dest: String,
    pub chmod: Option<u16>,
    pub chown: Option<Chown>,
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
        write!(f, "{:?} {:?}", self.src, self.dest)
    }
}
