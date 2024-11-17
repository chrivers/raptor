mod copy;
mod from;
mod include;
mod invoke;
mod render;
mod run;
mod write;

pub use copy::*;
pub use from::*;
pub use include::*;
pub use invoke::*;
pub use render::*;
pub use run::*;
pub use write::*;

use std::fmt::{Debug, Display};
use std::ops::Range;
use std::sync::Arc;

use crate::{RaptorError, RaptorResult};

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

#[derive(Clone)]
pub enum Instruction {
    From(InstFrom),
    Copy(InstCopy),
    Render(InstRender),
    Write(InstWrite),
    Include(InstInclude),
    Invoke(InstInvoke),
    Run(InstRun),
}

#[derive(Clone)]
pub struct Origin {
    pub path: Arc<String>,
    pub span: Range<usize>,
}

#[derive(Clone, Debug)]
pub struct Statement {
    pub inst: Instruction,
    pub origin: Origin,
}

impl Statement {
    pub fn error(&self, msg: String) -> RaptorResult<()> {
        Err(RaptorError::ScriptError(msg, self.origin.clone()))
    }
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
        }
    }
}

impl Debug for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{:<15} {:>3} .. {:>3}]",
            self.path, self.span.start, self.span.end
        )
    }
}
