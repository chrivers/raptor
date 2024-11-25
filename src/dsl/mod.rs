use std::fmt::{Debug, Display};

mod copy;
mod env;
mod from;
mod include;
mod invoke;
mod item;
mod origin;
mod render;
mod run;
mod workdir;
mod write;

pub use copy::*;
pub use env::*;
pub use from::*;
pub use include::*;
pub use invoke::*;
pub use item::*;
pub use origin::*;
pub use render::*;
pub use run::*;
pub use workdir::*;
pub use write::*;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Chown {
    pub user: Option<String>,
    pub group: Option<String>,
}

impl Chown {
    pub fn new(user: impl AsRef<str>, group: impl AsRef<str>) -> Self {
        Self {
            user: Some(user.as_ref().to_owned()),
            group: Some(group.as_ref().to_owned()),
        }
    }

    pub fn user(user: impl AsRef<str>) -> Self {
        Self {
            user: Some(user.as_ref().to_owned()),
            group: None,
        }
    }

    pub fn group(group: impl AsRef<str>) -> Self {
        Self {
            user: None,
            group: Some(group.as_ref().to_owned()),
        }
    }
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

#[derive(Clone, PartialEq, Eq)]
pub enum Instruction {
    From(InstFrom),
    Copy(InstCopy),
    Render(InstRender),
    Write(InstWrite),
    Include(InstInclude),
    Invoke(InstInvoke),
    Run(InstRun),
    Env(InstEnv),
    Workdir(InstWorkdir),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Statement {
    pub inst: Instruction,
    pub origin: Origin,
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
            Self::Run(_) => "RUN",
            Self::Env(_) => "ENV",
            Self::Workdir(_) => "WORKDIR",
        }
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::From(inst) => inst.fmt(f),
            Self::Copy(inst) => inst.fmt(f),
            Self::Render(inst) => inst.fmt(f),
            Self::Write(inst) => inst.fmt(f),
            Self::Include(inst) => inst.fmt(f),
            Self::Invoke(inst) => inst.fmt(f),
            Self::Run(inst) => inst.fmt(f),
            Self::Env(inst) => inst.fmt(f),
            Self::Workdir(inst) => inst.fmt(f),
        }
    }
}
