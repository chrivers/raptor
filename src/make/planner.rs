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

pub enum Job {
    Build(BuildLayer),
    Run(RunTarget),
}

pub struct Planner<'a> {
    pub nodes: HashMap<u64, Node<u64>>,
    pub jobs: HashMap<u64, Job>,
    maker: &'a Maker,
}

impl<'a> Planner<'a> {
    #[must_use]
    pub fn new(maker: &'a Maker) -> Self {
        Self {
            nodes: HashMap::new(),
            jobs: HashMap::new(),
            maker,
        }
    }

    pub fn add_build_job(
        &mut self,
        builder: &RaptorBuilder,
        input: &str,
    ) -> RaptorResult<Option<u64>> {
        let prog = builder.load(input)?;
        let targets = builder.stack(prog)?;

        let mut last = None;
        let mut layers = vec![];

        for st in targets.iter().rev() {
            let li = builder.layer_info(st)?;
            let hash = li.hash_value();
            let done_path = li.done_path();

            if !self.jobs.contains_key(&hash) {
                self.nodes.insert(hash, Node::new(hash));
                let job = Job::Build(BuildLayer::new(st, &layers, li));
                self.jobs.insert(hash, job);
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

        let filename = builder.loader().to_program_path(&job.target, &origin)?;
        let job_hash = self.add_build_job(builder, filename.as_str())?;

        let run_hash = job.hash_value();

        let mut node = Node::new(run_hash);
        job_hash.inspect(|job_hash| node.add_dep(*job_hash));
        self.nodes.insert(run_hash, node);
        self.jobs.insert(run_hash, Job::Run(job.clone()));

        for input in &job.input {
            let input_hash = self.add_build_job(builder, input)?;

            if let Some(input_hash) = input_hash {
                self.nodes
                    .entry(run_hash)
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
    pub fn into_plan(self) -> (DepGraph<u64>, HashMap<u64, Job>) {
        (
            DepGraph::new(&self.nodes.into_values().collect_vec()),
            self.jobs,
        )
    }
}
