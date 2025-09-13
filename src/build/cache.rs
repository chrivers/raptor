use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::os::unix::fs::MetadataExt;

use camino::Utf8PathBuf;
use itertools::Itertools;

use crate::dsl::{Instruction, Program};
use crate::{RaptorError, RaptorResult};

pub struct Cacher;

impl Cacher {
    pub fn cache_key(prog: &Program) -> RaptorResult<u64> {
        let mut state = DefaultHasher::new();

        for source in &Self::sources(prog)? {
            let md = source
                .metadata()
                .map_err(|err| RaptorError::CacheIoError(source.into(), err))?;

            md.ctime().hash(&mut state);
            md.ctime_nsec().hash(&mut state);
        }

        Ok(state.finish())
    }

    pub fn sources(prog: &Program) -> RaptorResult<Vec<Utf8PathBuf>> {
        let mut data = HashSet::<Utf8PathBuf>::new();

        data.insert(prog.path.clone());

        prog.traverse(&mut |stmt| {
            match &stmt.inst {
                Instruction::Copy(inst) => {
                    data.extend(
                        inst.srcs
                            .iter()
                            .map(|file| prog.path_for(file))
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                }

                Instruction::Render(inst) => {
                    data.insert(prog.path_for(&inst.src)?);
                }

                Instruction::Include(inst) => {
                    data.insert(prog.path_for(&inst.src)?);
                }

                Instruction::Invoke(_)
                | Instruction::Write(_)
                | Instruction::Mkdir(_)
                | Instruction::From(_)
                | Instruction::Run(_)
                | Instruction::Env(_)
                | Instruction::Workdir(_) => {}
            }

            Ok(())
        })?;

        Ok(data.into_iter().sorted().collect())
    }
}

pub struct LayerInfo {
    name: String,
    hash: u64,
}

impl LayerInfo {
    #[must_use]
    pub const fn new(name: String, hash: u64) -> Self {
        Self { name, hash }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn hash(&self) -> String {
        format!("{:016X}", self.hash)
    }

    #[must_use]
    pub fn id(&self) -> String {
        format!("{}-{:016X}", self.name, self.hash)
    }

    #[must_use]
    pub fn work_path(&self) -> String {
        format!("layers/build-{}", self.id())
    }

    #[must_use]
    pub fn done_path(&self) -> String {
        format!("layers/{}", self.id())
    }
}
