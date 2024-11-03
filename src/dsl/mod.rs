mod copy;
mod render;

pub use copy::*;
pub use render::*;

use std::fmt::{Debug, Display};

pub struct Chown {
    pub user: Option<String>,
    pub group: Option<String>,
}

impl Display for Chown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(user) = &self.user {
            write!(f, "{user}")?;
        }
        if let Some(grp) = &self.group {
            write!(f, ":{grp}")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct InstFrom {
    pub from: String,
}

#[derive(Debug)]
pub enum Instruction {
    From(InstFrom),
    Copy(InstCopy),
    Render(InstRender),
}
