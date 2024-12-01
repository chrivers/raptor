use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use itertools::Itertools;

use crate::dsl::{Instruction, Program};
use crate::{RaptorError, RaptorResult};

pub struct Cacher;

impl Cacher {
    pub fn cache_key(prog: &Program) -> RaptorResult<u64> {
        let mut state = DefaultHasher::new();
        prog.hash(&mut state);

        for source in &Self::sources(prog)? {
            let md = source
                .metadata()
                .map_err(|err| RaptorError::CacheIoError(source.into(), err))?;

            md.ctime().hash(&mut state);
            md.ctime_nsec().hash(&mut state);
        }

        Ok(state.finish())
    }

    #[must_use]
    pub fn layer_info(program: &Program, hash: u64) -> LayerInfo {
        LayerInfo::new(program, hash)
    }

    #[must_use]
    pub fn sources(prog: &Program) -> Vec<String> {
        let mut res = HashSet::<String>::new();
        let data = &mut res;
        prog.traverse(&mut |stmt| match &stmt.inst {
            Instruction::Copy(inst) => data.extend(inst.srcs.clone()),
            Instruction::Render(inst) => {
                data.insert(inst.src.clone());
            }
            Instruction::Include(_)
            | Instruction::Invoke(_)
            | Instruction::Write(_)
            | Instruction::From(_)
            | Instruction::Run(_)
            | Instruction::Env(_)
            | Instruction::Workdir(_) => {}
        });

        res.into_iter().sorted().collect()
    }
}

pub struct LayerInfo {
    name: String,
    hash: u64,
}

impl LayerInfo {
    #[must_use]
    pub fn new(program: &Program, hash: u64) -> Self {
        let name = program.path.file_stem().unwrap().into();
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
