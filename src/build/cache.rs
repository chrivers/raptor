use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::os::unix::fs::MetadataExt;
use std::sync::Arc;

use camino::{Utf8Path, Utf8PathBuf};
use itertools::Itertools;
use raptor_parser::util::module_name::ModuleRoot;

use crate::build::RaptorBuilder;
use crate::dsl::Program;
use crate::program::Loader;
use crate::{RaptorError, RaptorResult};
use raptor_parser::ast::{FromSource, Instruction};

pub struct Cacher;

impl Cacher {
    pub fn cache_key(program: &Arc<Program>, builder: &mut RaptorBuilder<'_>) -> RaptorResult<u64> {
        let mut state = DefaultHasher::new();

        if let Some((from, origin)) = program.from() {
            match from {
                FromSource::Raptor(from) => {
                    let filename = builder.loader().to_program_path(from, origin)?;

                    let prog = builder.load_with_source(filename, origin.clone())?;
                    Self::cache_key(&prog, builder)?.hash(&mut state);
                }
                FromSource::Docker(src) => src.hash(&mut state),
            }
        }

        for stmt in &program.code {
            stmt.hash(&mut state);
        }

        for source in &Self::sources(program, builder.loader())? {
            trace!("Checking source [{source}]");
            let md = source
                .metadata()
                .map_err(|err| RaptorError::CacheIoError(source.into(), err))?;

            md.ctime().hash(&mut state);
            md.ctime_nsec().hash(&mut state);
        }

        Ok(state.finish())
    }

    pub fn sources(prog: &Program, loader: &Loader) -> RaptorResult<Vec<Utf8PathBuf>> {
        let mut data = HashSet::<Utf8PathBuf>::new();

        prog.traverse(&mut |stmt| {
            match &stmt.inst {
                Instruction::Copy(inst) => {
                    data.extend(
                        inst.srcs
                            .iter()
                            .map(|file| loader.to_path(&ModuleRoot::Relative, &stmt.origin, file))
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                }

                Instruction::Render(inst) => {
                    data.insert(prog.path_for(&inst.src)?);
                }

                Instruction::Include(inst) => {
                    let path = loader.to_include_path(&inst.src, &stmt.origin)?;
                    data.insert(path);
                }

                Instruction::Mount(_)
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
