use std::collections::HashMap;
use std::process::ExitStatus;
use std::time::SystemTime;

use camino::{Utf8Path, Utf8PathBuf};
use itertools::Itertools;
use raptor_parser::ast::Origin;

use crate::build::{BuildTarget, Cacher, RaptorBuilder};
use crate::dsl::Program;
use crate::make::parser::Make;
use crate::program::Loader;
use crate::runner::Runner;
use crate::{RaptorError, RaptorResult};

pub struct Maker {
    make: Make,
}

impl Maker {
    pub fn load(path: &Utf8Path) -> RaptorResult<Self> {
        let text = std::fs::read_to_string(path)?;
        let make: Make = toml::from_str(&text)?;
        Ok(Self { make })
    }

    #[must_use]
    pub const fn rules(&self) -> &Make {
        &self.make
    }

    pub fn add_links(&self, loader: &mut Loader) {
        for (name, link) in &self.make.raptor.link {
            loader.add_package(name.to_string(), Utf8PathBuf::from(&link.source));
        }
    }

    fn program_mtime(program: &Program, loader: &Loader) -> RaptorResult<SystemTime> {
        let sources = Cacher::sources(program, loader)?;

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

    pub fn run_job(&self, builder: &mut RaptorBuilder, name: &str) -> RaptorResult<ExitStatus> {
        let job = self
            .make
            .run
            .get(name)
            .ok_or_else(|| RaptorError::UnknownJob(name.to_string()))?;

        let origin = Origin::make("<command-line>", 0..0);
        let filename = builder.loader().to_program_path(&job.target, &origin)?;

        let program = builder.load(filename)?;

        let mut newest = Self::program_mtime(&program, builder.loader())?;

        for input in &job.input {
            let prog = builder.load(input)?;
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

        if let Some(output) = &job.output
            && Utf8Path::new(output)
                .metadata()
                .and_then(|md| md.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH)
                >= newest
        {
            info!("Target [{name}] up to date");
            return Ok(ExitStatus::default());
        }

        builder.build_program(program.clone())?;

        let mut layers = vec![];

        for target in builder.stack(program.clone())? {
            layers.push(target.layer_info(builder)?.done_path());
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

        if let Some(output) = &job.output {
            mounts.insert("output", vec![output.as_str()]);
        }

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

    pub fn run_group(&self, builder: &mut RaptorBuilder, name: &str) -> RaptorResult<()> {
        let group = self
            .make
            .group
            .get(name)
            .ok_or_else(|| RaptorError::UnknownJob(name.to_string()))?;

        for run in &group.run {
            self.run_job(builder, run)?;
        }

        Ok(())
    }
}
