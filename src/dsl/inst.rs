use std::fmt::Debug;

use crate::dsl::{
    InstCopy, InstEnv, InstFrom, InstInclude, InstInvoke, InstRender, InstRun, InstWorkdir,
    InstWrite,
};

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
