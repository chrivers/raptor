use std::collections::HashMap;

use camino::Utf8PathBuf;
use dep_graph::{DepGraph, Node};
use itertools::Itertools;
use raptor_parser::ast::Origin;

use crate::RaptorResult;
use crate::build::{BuildTarget, LayerInfo, RaptorBuilder};
use crate::make::parser::RunTarget;

#[derive(Debug)]
pub struct Work {
    pub layers: Vec<Utf8PathBuf>,
    pub target: BuildTarget,
    pub layerinfo: LayerInfo,
}

impl Work {
    #[must_use]
    pub fn new(target: &BuildTarget, layers: &[Utf8PathBuf], layerinfo: LayerInfo) -> Self {
        Self {
            layers: layers.to_vec(),
            target: target.clone(),
            layerinfo,
        }
    }
}

#[derive(Default)]
pub struct Planner {
    pub nodes: HashMap<u64, Node<u64>>,
    pub targets: HashMap<u64, Work>,
}

impl Planner {
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            targets: HashMap::new(),
        }
    }

    pub fn add_target(
        &mut self,
        builder: &mut RaptorBuilder,
        targets: &[BuildTarget],
    ) -> RaptorResult<Option<u64>> {
        let mut last = None;
        let mut layers = vec![];
        for st in targets {
            let li = st.layer_info(builder)?;
            let hash = li.hash_value();
            let done_path = li.done_path();

            if !self.targets.contains_key(&hash) {
                self.nodes.insert(hash, Node::new(hash));
                self.targets.insert(hash, Work::new(st, &layers, li));
            }
            let work = self.nodes.get_mut(&hash).unwrap();

            match &st {
                BuildTarget::Program(_program) => {
                    last.inspect(|id| work.add_dep(*id));
                }
                BuildTarget::DockerSource(_) => {}
            }

            layers.push(done_path);
            last = Some(hash);
        }

        Ok(last)
    }

    pub fn add_job(&mut self, builder: &mut RaptorBuilder, job: &RunTarget) -> RaptorResult<()> {
        let origin = Origin::make("<command-line>", 0..0);

        let name = &job.target;
        let filename = builder.loader().to_program_path(name, &origin)?;
        let prog = builder.load(&filename)?;
        let stack = builder.stack(prog)?;
        let job_hash = self.add_target(builder, &stack)?;

        for input in &job.input {
            let prog = builder.load(input)?;
            let stack = builder.stack(prog)?;

            let input_hash = self.add_target(builder, &stack)?;

            if let Some((input_hash, job_hash)) = input_hash.zip(job_hash) {
                self.nodes
                    .entry(job_hash)
                    .and_modify(|node| node.add_dep(input_hash));
            }
        }

        Ok(())
    }

    #[must_use]
    pub fn into_plan(self) -> (DepGraph<u64>, HashMap<u64, Work>) {
        (
            DepGraph::new(&self.nodes.into_values().collect_vec()),
            self.targets,
        )
    }
}
