use std::fmt::{Debug, Display};

use camino::Utf8Path;

use crate::ast::{
    Chown, IncludeArg, InstCmd, InstCopy, InstEntrypoint, InstEnv, InstEnvAssign, InstFrom,
    InstInclude, InstMkdir, InstMount, InstRender, InstRun, InstWorkdir, InstWrite,
};
use crate::util::module_name::ModuleName;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Instruction {
    From(InstFrom),
    Mount(InstMount),
    Copy(InstCopy),
    Render(InstRender),
    Write(InstWrite),
    Mkdir(InstMkdir),
    Include(InstInclude),
    Run(InstRun),
    Env(InstEnv),
    Workdir(InstWorkdir),
    Entrypoint(InstEntrypoint),
    Cmd(InstCmd),
}

impl Instruction {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::From(_) => "FROM",
            Self::Mount(_) => "MOUNT",
            Self::Copy(_) => "COPY",
            Self::Render(_) => "RENDER",
            Self::Write(_) => "WRITE",
            Self::Mkdir(_) => "MKDIR",
            Self::Include(_) => "INCLUDE",
            Self::Run(_) => "RUN",
            Self::Env(_) => "ENV",
            Self::Workdir(_) => "WORKDIR",
            Self::Entrypoint(_) => "ENTRYPOINT",
            Self::Cmd(_) => "CMD",
        }
    }

    pub fn include(src: impl AsRef<str>, args: impl IntoIterator<Item = IncludeArg>) -> Self {
        Self::Include(InstInclude {
            src: ModuleName::from(src.as_ref()),
            args: args.into_iter().collect(),
        })
    }

    pub fn workdir(dir: impl AsRef<Utf8Path>) -> Self {
        Self::Workdir(InstWorkdir {
            dir: dir.as_ref().to_path_buf(),
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

    pub fn copy(srcs: &[impl AsRef<Utf8Path>], dest: impl AsRef<str>) -> Self {
        Self::Copy(InstCopy {
            chmod: None,
            chown: None,
            srcs: srcs.iter().map(|s| s.as_ref().to_path_buf()).collect(),
            dest: dest.as_ref().into(),
        })
    }

    pub fn write(body: impl AsRef<str>, dest: impl AsRef<Utf8Path>) -> Self {
        Self::Write(InstWrite {
            dest: dest.as_ref().to_path_buf(),
            body: body.as_ref().to_string(),
            chmod: None,
            chown: None,
        })
    }

    pub fn mkdir(dest: impl AsRef<Utf8Path>) -> Self {
        Self::Mkdir(InstMkdir {
            dest: dest.as_ref().to_path_buf(),
            chmod: None,
            chown: None,
            parents: false,
        })
    }

    pub fn render(
        src: impl AsRef<Utf8Path>,
        dest: impl AsRef<Utf8Path>,
        args: impl IntoIterator<Item = IncludeArg>,
    ) -> Self {
        Self::Render(InstRender {
            src: src.as_ref().to_path_buf(),
            dest: dest.as_ref().to_path_buf(),
            args: args.into_iter().collect(),
            chmod: None,
            chown: None,
        })
    }

    pub fn entrypoint(args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::Entrypoint(InstEntrypoint {
            entrypoint: args.into_iter().map(Into::into).collect(),
        })
    }

    pub fn cmd(args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::Cmd(InstCmd {
            cmd: args.into_iter().map(Into::into).collect(),
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

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::From(inst) => Display::fmt(inst, f),
            Self::Mount(inst) => Display::fmt(inst, f),
            Self::Copy(inst) => Display::fmt(inst, f),
            Self::Render(inst) => Display::fmt(inst, f),
            Self::Write(inst) => Display::fmt(inst, f),
            Self::Mkdir(inst) => Display::fmt(inst, f),
            Self::Include(inst) => Display::fmt(inst, f),
            Self::Run(inst) => Display::fmt(inst, f),
            Self::Env(inst) => Display::fmt(inst, f),
            Self::Workdir(inst) => Display::fmt(inst, f),
            Self::Entrypoint(inst) => Display::fmt(inst, f),
            Self::Cmd(inst) => Display::fmt(inst, f),
        }
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::From(inst) => Debug::fmt(inst, f),
            Self::Mount(inst) => Debug::fmt(inst, f),
            Self::Copy(inst) => Debug::fmt(inst, f),
            Self::Render(inst) => Debug::fmt(inst, f),
            Self::Write(inst) => Debug::fmt(inst, f),
            Self::Mkdir(inst) => Debug::fmt(inst, f),
            Self::Include(inst) => Debug::fmt(inst, f),
            Self::Run(inst) => Debug::fmt(inst, f),
            Self::Env(inst) => Debug::fmt(inst, f),
            Self::Workdir(inst) => Debug::fmt(inst, f),
            Self::Entrypoint(inst) => Debug::fmt(inst, f),
            Self::Cmd(inst) => Debug::fmt(inst, f),
        }
    }
}
