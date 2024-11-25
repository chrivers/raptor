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

    pub fn write(
        dest: impl AsRef<str>,
        body: impl AsRef<str>,
        chmod: Option<u32>,
        chown: Option<Chown>,
    ) -> Self {
        Self::Write(InstWrite {
            dest: dest.as_ref().to_string(),
            body: body.as_ref().to_string(),
            chmod,
            chown,
        })
    }

    pub fn render(
        src: impl AsRef<str>,
        dest: impl AsRef<str>,
        chmod: Option<u32>,
        chown: Option<Chown>,
        args: impl IntoIterator<Item = IncludeArg>,
    ) -> Self {
        Self::Render(InstRender {
            src: src.as_ref().to_string(),
            dest: dest.as_ref().to_string(),
            args: args.into_iter().collect(),
            chmod,
            chown,
        })
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
