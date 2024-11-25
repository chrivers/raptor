use std::fmt::Debug;

use crate::dsl::{
    Chown, IncludeArg, InstCopy, InstEnv, InstEnvAssign, InstFrom, InstInclude, InstInvoke,
    InstRender, InstRun, InstWorkdir, InstWrite,
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

    pub fn workdir(dir: impl AsRef<str>) -> Self {
        Self::Workdir(InstWorkdir {
            dir: dir.as_ref().to_string(),
        })
    }

    #[must_use]
    pub fn env(env: impl IntoIterator<Item = InstEnvAssign>) -> Self {
        Self::Env(InstEnv {
            env: env.into_iter().collect(),
        })
    }

    pub fn run(run: &[impl AsRef<str>]) -> Self {
        Self::Run(InstRun {
            run: run.iter().map(|s| s.as_ref().to_string()).collect(),
        })
    }

    pub fn write(dest: impl AsRef<str>, body: impl AsRef<str>) -> Self {
        Self::Write(InstWrite {
            dest: dest.as_ref().to_string(),
            body: body.as_ref().to_string(),
            chmod: None,
            chown: None,
        })
    }

    pub fn render(
        src: impl AsRef<str>,
        dest: impl AsRef<str>,
        args: impl IntoIterator<Item = IncludeArg>,
    ) -> Self {
        Self::Render(InstRender {
            src: src.as_ref().to_string(),
            dest: dest.as_ref().to_string(),
            args: args.into_iter().collect(),
            chmod: None,
            chown: None,
        })
    }

    #[must_use]
    pub fn chmod(self, chmod: Option<u32>) -> Self {
        match self {
            Self::Copy(inst) => Self::Copy(InstCopy { chmod, ..inst }),
            Self::Write(inst) => Self::Write(InstWrite { chmod, ..inst }),
            Self::Render(inst) => Self::Render(InstRender { chmod, ..inst }),
            _ => self,
        }
    }

    #[must_use]
    pub fn chown(self, chown: Option<Chown>) -> Self {
        match self {
            Self::Copy(inst) => Self::Copy(InstCopy { chown, ..inst }),
            Self::Write(inst) => Self::Write(InstWrite { chown, ..inst }),
            Self::Render(inst) => Self::Render(InstRender { chown, ..inst }),
            _ => self,
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
