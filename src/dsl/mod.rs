mod copy;
mod include;
mod invoke;
mod render;
mod write;

pub use copy::*;
pub use include::*;
pub use invoke::*;
pub use render::*;
pub use write::*;

use std::fmt::{Debug, Display};

#[derive(Clone)]
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

#[derive(Clone, Debug)]
pub struct InstFrom {
    pub from: String,
}

#[derive(Clone, Debug)]
pub enum Instruction {
    From(InstFrom),
    Copy(InstCopy),
    Render(InstRender),
    Write(InstWrite),
    Include(InstInclude),
    Invoke(InstInvoke),
}

impl Instruction {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::From(_) => "FROM",
            Self::Copy(_) => "COPY",
            Self::Render(_) => "RENDER",
            Self::Write(_) => "WRITE",
            Self::Include(_) => "INCLUDE",
            Self::Invoke(_) => "INVOKE",
        }
    }
}
