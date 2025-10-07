use std::collections::HashMap;

use camino::Utf8PathBuf;
use dep_graph::{DepGraph, Node};
use itertools::Itertools;
use raptor_parser::ast::Origin;

use crate::RaptorResult;
use crate::build::{BuildTarget, LayerInfo, RaptorBuilder};
use crate::make::maker::Maker;
use crate::make::parser::{MakeTarget, RunTarget};

#[derive(Debug)]
pub struct BuildLayer {
    pub layers: Vec<Utf8PathBuf>,
    pub target: BuildTarget,
    pub layerinfo: LayerInfo,
}

impl BuildLayer {
    #[must_use]
    pub fn new(target: &BuildTarget, layers: &[Utf8PathBuf], layerinfo: LayerInfo) -> Self {
        Self {
            layers: layers.to_vec(),
            target: target.clone(),
            layerinfo,
        }
    }
}

pub struct Planner<'a> {
    pub nodes: HashMap<u64, Node<u64>>,
    pub targets: HashMap<u64, BuildLayer>,
    maker: &'a Maker,
}

impl<'a> Planner<'a> {
    #[must_use]
    pub fn new(maker: &'a Maker) -> Self {
        Self {
            nodes: HashMap::new(),
            targets: HashMap::new(),
            maker,
        }
    }

    pub fn add_build_job(
        &mut self,
        builder: &RaptorBuilder,
        targets: &[BuildTarget],
    ) -> RaptorResult<Option<u64>> {
        let mut last = None;
        let mut layers = vec![];
        for st in targets {
            let li = builder.layer_info(st)?;
            let hash = li.hash_value();
            let done_path = li.done_path();

            if !self.targets.contains_key(&hash) {
                self.nodes.insert(hash, Node::new(hash));
                self.targets.insert(hash, BuildLayer::new(st, &layers, li));
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

    pub fn add_run_job(&mut self, builder: &RaptorBuilder, job: &RunTarget) -> RaptorResult<()> {
        let origin = Origin::make("<command-line>", 0..0);

        let name = &job.target;
        let filename = builder.loader().to_program_path(name, &origin)?;
        let prog = builder.load(&filename)?;
        let stack = builder.stack(prog)?;
        let job_hash = self.add_build_job(builder, &stack)?;

        for input in &job.input {
            let prog = builder.load(input)?;
            let stack = builder.stack(prog)?;

            let input_hash = self.add_build_job(builder, &stack)?;

            if let Some((input_hash, job_hash)) = input_hash.zip(job_hash) {
                self.nodes
                    .entry(job_hash)
                    .and_modify(|node| node.add_dep(input_hash));
            }
        }

        Ok(())
    }

    pub fn add(&mut self, builder: &RaptorBuilder, target: &MakeTarget) -> RaptorResult<()> {
        match target {
            MakeTarget::Group(grp) => {
                for run in &self.maker.rules().group[grp].run {
                    let job = &self.maker.rules().run[run];
                    self.add_run_job(builder, job)?;
                }
            }
            MakeTarget::Job(job) => {
                let job = &self.maker.rules().run[job];
                self.add_run_job(builder, job)?;
            }
        }

        Ok(())
    }

    #[must_use]
    pub fn into_plan(self) -> (DepGraph<u64>, HashMap<u64, BuildLayer>) {
        (
            DepGraph::new(&self.nodes.into_values().collect_vec()),
            self.targets,
        )
    }
}
