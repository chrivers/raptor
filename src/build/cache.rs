use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use itertools::Itertools;

use crate::dsl::{Instruction, Program};
use crate::RaptorResult;

pub struct Cacher;

impl Cacher {
    pub fn cache_key(prog: &Program) -> RaptorResult<u64> {
        let mut state = DefaultHasher::new();
        prog.hash(&mut state);

        for source in &Self::sources(prog) {
            let md = Path::new(source).metadata()?;

            md.ctime().hash(&mut state);
            md.ctime_nsec().hash(&mut state);
        }

        Ok(state.finish())
    }

    #[must_use]
    pub fn layer_name(prog: &Program, hash: u64) -> String {
        format!("{}-{hash:016X}", prog.path.file_stem().unwrap())
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
