use std::collections::HashMap;
use std::process::ExitStatus;
use std::time::SystemTime;

use camino::{Utf8Path, Utf8PathBuf};
use itertools::Itertools;
use raptor_parser::util::module_name::ModuleName;

use crate::build::{BuildTarget, Cacher, RaptorBuilder};
use crate::dsl::Program;
use crate::make::parser::{Make, MakeTarget, RunTarget};
use crate::make::planner::BuildLayer;
use crate::program::Loader;
use crate::runner::Runner;
use crate::{RaptorError, RaptorResult};

pub struct Maker<'a> {
    make: Make,
    builder: &'a RaptorBuilder<'a>,
}

impl<'a> Maker<'a> {
    pub fn load(builder: &'a RaptorBuilder, path: &Utf8Path) -> RaptorResult<Self> {
        let text = std::fs::read_to_string(path)?;
        let make: Make = toml::from_str(&text)?;
        Ok(Self { make, builder })
    }

    #[must_use]
    pub const fn builder(&self) -> &RaptorBuilder {
        self.builder
    }

    #[must_use]
    pub const fn rules(&self) -> &Make {
        &self.make
    }

    pub fn add_links(&self, loader: &Loader) {
        for (name, link) in &self.make.raptor.link {
            loader.add_package(name.to_string(), Utf8PathBuf::from(&link.source));
        }
    }

    fn program_mtime(program: &Program, _loader: &Loader) -> RaptorResult<SystemTime> {
        let sources = Cacher::sources(program)?;

        let res = sources
            .into_iter()
            .map(|source| {
                Ok(source
                    .metadata()
                    .map_err(|err| RaptorError::CacheIoError(source, err))?
                    .modified()?)
            })
            .collect::<Result<Vec<_>, RaptorError>>()?
            .into_iter()
            .max()
            .unwrap_or(SystemTime::UNIX_EPOCH);

        Ok(res)
    }

    pub fn run_named_job(&self, name: &str) -> RaptorResult<ExitStatus> {
        let job = self
            .make
            .run
            .get(name)
            .ok_or_else(|| RaptorError::UnknownJob(name.to_string()))?;

        self.run_job(job)
    }

    pub fn run_job(&self, job: &RunTarget) -> RaptorResult<ExitStatus> {
        let builder = self.builder;

        let program = builder.load(&job.target)?;

        let mut newest = Self::program_mtime(&program, builder.loader())?;

        for input in &job.input {
            let prog = builder.load(&ModuleName::from(input))?;
            let stack = builder.stack(prog)?;
            for st in stack {
                match st {
                    BuildTarget::Program(program) => {
                        newest = newest.max(Self::program_mtime(&program, builder.loader())?);
                    }
                    BuildTarget::DockerSource(_) => {}
                }
            }
        }

        let oldest = job
            .output
            .iter()
            .map(Utf8Path::new)
            .flat_map(Utf8Path::metadata)
            .flat_map(|md| md.modified())
            .min()
            .unwrap_or(SystemTime::UNIX_EPOCH);

        if oldest >= newest {
            info!("Output is up to date");
            return Ok(ExitStatus::default());
        }

        builder.build_program(program.clone())?;

        let mut layers = vec![];

        for target in builder.stack(program.clone())? {
            layers.push(builder.layer_info(&target)?.done_path());
        }

        let mut mounts = HashMap::<&str, Vec<&str>>::new();

        mounts
            .entry("cache")
            .or_default()
            .extend(job.cache.iter().map(String::as_str));

        mounts
            .entry("input")
            .or_default()
            .extend(job.input.iter().map(String::as_str));

        mounts
            .entry("output")
            .or_default()
            .extend(job.output.iter().map(String::as_str));

        let env = job
            .env
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect_vec();

        let mut runner = Runner::new()?;
        runner
            .with_mounts(mounts)
            .with_env(&env)
            .with_args(&job.args);

        if !job.entrypoint.is_empty() {
            runner.with_entrypoint(&job.entrypoint);
        }

        if let Some(state_dir) = &job.state_dir {
            runner.with_state_dir(state_dir.into());
        }

        let res = runner.spawn(&program, builder, &layers)?;

        Ok(res)
    }

    pub fn run_group(&self, name: &str) -> RaptorResult<()> {
        let group = self
            .make
            .group
            .get(name)
            .ok_or_else(|| RaptorError::UnknownJob(name.to_string()))?;

        for run in &group.run {
            self.run_named_job(run)?;
        }

        Ok(())
    }

    pub fn run(&self, target: &MakeTarget) -> RaptorResult<()> {
        match target {
            MakeTarget::Job(job) => self.run_named_job(job).map(|_| ()),
            MakeTarget::Group(grp) => self.run_group(grp),
        }
    }

    pub fn build(&self, build: &BuildLayer) -> RaptorResult<Utf8PathBuf> {
        self.builder()
            .build_layer(&build.layers, &build.target, &build.layerinfo)
    }
}
