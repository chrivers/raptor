use std::fmt::{Debug, Display};

use crate::dsl::Chown;
use crate::print::Theme;

#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub struct InstWrite {
    pub dest: String,
    pub body: String,
    pub chmod: Option<u32>,
    pub chown: Option<Chown>,
}

impl Display for InstWrite {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.keyword("WRITE")?;
        f.chmod(&self.chmod)?;
        f.chown(&self.chown)?;
        f.dest(&self.dest)?;
        f.value(&self.body)?;
        Ok(())
    }
}
