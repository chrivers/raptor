use std::collections::HashSet;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::sync::Arc;

use camino::{Utf8Path, Utf8PathBuf};
use itertools::Itertools;
use siphasher::sip::SipHasher13;

use crate::build::{BuildTarget, RaptorBuilder};
use crate::dsl::{Item, Program};
use crate::{RaptorError, RaptorResult};
use raptor_parser::ast::{FromSource, Instruction, Statement};

pub struct Cacher;

impl Cacher {
    fn hash_file(path: &Utf8Path, state: &mut impl Hasher) -> RaptorResult<()> {
        let mut file = File::open(path)?;

        let mut buf = vec![0; 128 * 1024];
        loop {
            match file.read(&mut buf)? {
                0 => break,
                n => &buf[..n].hash(state),
            };
        }

        Ok(())
    }

    const fn include_in_build_hash(stmt: &Statement) -> bool {
        match stmt.inst {
            Instruction::Copy(_)
            | Instruction::Render(_)
            | Instruction::Write(_)
            | Instruction::Mkdir(_)
            | Instruction::Run(_)
            | Instruction::Env(_)
            | Instruction::Workdir(_) => true,

            Instruction::From(_)
            | Instruction::Mount(_)
            | Instruction::Include(_)
            | Instruction::Entrypoint(_)
            | Instruction::Cmd(_) => false,
        }
    }

    fn flatten_program<'a>(program: &'a Program, out: &mut Vec<&'a Statement>) {
        for item in &program.code {
            match item {
                Item::Statement(stmt) => out.push(stmt),
                Item::Program(prgm) => Self::flatten_program(prgm, out),
            }
        }
    }

    pub fn cache_key(program: &Arc<Program>, builder: &RaptorBuilder<'_>) -> RaptorResult<u64> {
        let mut state = SipHasher13::new();

        if let Some((from, origin)) = program.from() {
            match from {
                FromSource::Raptor(from) => {
                    let prog = builder.loader().load_program(from, origin.clone())?;
                    Self::cache_key(&prog, builder)?.hash(&mut state);
                }
                FromSource::Docker(src) => src.hash(&mut state),
            }
        }

        let mut code = vec![];
        Self::flatten_program(program, &mut code);

        code.iter()
            .as_ref()
            .iter()
            .filter(|stmt| Self::include_in_build_hash(stmt))
            .for_each(|stmt| stmt.inst.hash(&mut state));

        for source in &Self::sources(program)? {
            trace!("Checking source [{source}]");
            let path = builder.loader().base().join(source);
            Self::hash_file(&path, &mut state)?;
        }

        Ok(state.finish())
    }

    pub fn sources(prog: &Program) -> RaptorResult<Vec<Utf8PathBuf>> {
        let mut data = HashSet::<Utf8PathBuf>::new();

        prog.traverse(&mut |stmt| {
            match &stmt.inst {
                Instruction::Copy(inst) => {
                    data.extend(
                        inst.srcs
                            .iter()
                            .map(|file| Ok(stmt.origin.path_for(file)?))
                            .collect::<Result<Vec<_>, RaptorError>>()?,
                    );
                }

                Instruction::Render(inst) => {
                    data.insert(stmt.origin.path_for(&inst.src)?);
                }

                Instruction::Include(_)
                | Instruction::Mount(_)
                | Instruction::Write(_)
                | Instruction::Mkdir(_)
                | Instruction::From(_)
                | Instruction::Run(_)
                | Instruction::Env(_)
                | Instruction::Workdir(_)
                | Instruction::Entrypoint(_)
                | Instruction::Cmd(_) => {}
            }

            Ok(())
        })?;

        Ok(data.into_iter().sorted().collect())
    }

    pub fn all_sources(prog: &Program, builder: &RaptorBuilder) -> RaptorResult<Vec<Utf8PathBuf>> {
        let base = builder.loader().base();
        let mut data: Vec<_> = Self::sources(prog)?.iter().map(|x| base.join(x)).collect();

        data.push(base.join(&prog.path));

        prog.traverse(&mut |stmt| {
            match &stmt.inst {
                Instruction::Include(inst) => {
                    let path = builder
                        .loader()
                        .resolver()
                        .to_include_path(&inst.src, &stmt.origin)?;
                    let path = base.join(path);
                    data.push(path);
                }

                Instruction::From(inst) => match &inst.from {
                    FromSource::Raptor(from) => {
                        let path = builder
                            .loader()
                            .resolver()
                            .to_program_path(from, &stmt.origin)?;
                        let path = base.join(path);
                        data.push(path);
                    }
                    FromSource::Docker(src) => {
                        let source = RaptorBuilder::parse_docker_source(src)?;
                        let info = builder.layer_info(&BuildTarget::DockerSource(source))?;
                        data.push(info.done_path());
                    }
                },

                _ => {}
            }

            Ok(())
        })?;

        Ok(data.into_iter().sorted().collect())
    }
}

#[derive(Debug, Clone)]
pub struct LayerInfo {
    name: String,
    hash: u64,
}

impl LayerInfo {
    pub const HASH_WIDTH: usize = 16;

    #[must_use]
    pub const fn new(name: String, hash: u64) -> Self {
        Self { name, hash }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn hash_value(&self) -> u64 {
        self.hash
    }

    #[must_use]
    pub fn hash(&self) -> String {
        format!("{:0width$X}", self.hash, width = Self::HASH_WIDTH)
    }

    #[must_use]
    pub fn id(&self) -> String {
        format!(
            "{}-{:0width$X}",
            self.name,
            self.hash,
            width = Self::HASH_WIDTH
        )
    }

    #[must_use]
    pub fn work_path(&self) -> Utf8PathBuf {
        Utf8Path::new("layers").join(format!("build-{}", self.id()))
    }

    #[must_use]
    pub fn done_path(&self) -> Utf8PathBuf {
        Utf8Path::new("layers").join(self.id())
    }
}

impl TryFrom<&str> for LayerInfo {
    type Error = RaptorError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let Some((head, tail)) = value.rsplit_once('-') else {
            return Err(RaptorError::LayerCacheParseError);
        };

        if tail.len() != Self::HASH_WIDTH {
            return Err(RaptorError::LayerCacheParseError);
        }

        let name = head.to_string();
        let hash = u64::from_str_radix(tail, 16)?;

        Ok(Self::new(name, hash))
    }
}

#[cfg(test)]
mod tests {
    use crate::build::LayerInfo;

    #[test]
    fn layerinfo_format() {
        let info = LayerInfo::new("name".to_string(), 0x0123_4567_89AB_CDEF);
        assert_eq!(info.name(), "name");
        assert_eq!(info.hash(), "0123456789ABCDEF");
        assert_eq!(info.id(), "name-0123456789ABCDEF");
    }

    #[test]
    fn layerinfo_parse() {
        let res = LayerInfo::try_from("name-0123456789ABCDEF").unwrap();
        assert_eq!(res.name, "name");
        assert_eq!(res.hash, 0x0123_4567_89AB_CDEF);

        LayerInfo::try_from("name-123456789ABCDEF").unwrap_err();
        LayerInfo::try_from("name-0123456789ABCDEF0").unwrap_err();
    }
}
