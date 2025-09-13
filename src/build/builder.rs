use std::collections::HashMap;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::process::Command;
use std::sync::Arc;

use camino::{Utf8Path, Utf8PathBuf};
use colored::Colorize;
use dregistry::downloader::DockerDownloader;
use dregistry::source::DockerSource;
use minijinja::context;

use crate::build::{Cacher, LayerInfo};
use crate::dsl::{FromSource, Program};
use crate::program::{Executor, Loader, PrintExecutor};
use crate::sandbox::Sandbox;
use crate::util::SafeParent;
use crate::RaptorResult;

pub struct RaptorBuilder<'a> {
    loader: Loader<'a>,
    dry_run: bool,
    programs: HashMap<Utf8PathBuf, Arc<Program>>,
}

#[derive(Debug)]
pub enum BuildTarget {
    Program(Arc<Program>),
    DockerSource(DockerSource),
}

trait DockerSourceExt {
    fn safe_file_name(&self) -> RaptorResult<Utf8PathBuf>;
}

impl DockerSourceExt for DockerSource {
    fn safe_file_name(&self) -> RaptorResult<Utf8PathBuf> {
        let safe = Self {
            host: Some(
                self.host
                    .as_deref()
                    .unwrap_or("index.docker.io")
                    .to_string(),
            ),
            port: self.port,
            namespace: Some(self.namespace.as_deref().unwrap_or("library").to_string()),
            repository: self.repository.clone(),
            tag: self.tag.clone(),
            digest: self.digest.clone(),
        };

        Ok(safe.to_string().replace(['/', ':'], "-").into())
    }
}

impl BuildTarget {
    pub fn layer_info(&self) -> RaptorResult<LayerInfo> {
        let name;
        let hash;

        match self {
            Self::Program(prog) => {
                debug!("Calculating hash for layer {}", prog.path);

                name = prog.path.file_stem().unwrap().into();
                hash = Cacher::cache_key(prog)?;
            }

            Self::DockerSource(image) => {
                debug!("Calculating hash for image {image}");

                name = image.safe_file_name()?;

                let mut state = DefaultHasher::new();
                image.hash(&mut state);
                hash = state.finish();
            }
        }

        Ok(LayerInfo::new(name.to_string(), hash))
    }

    fn simulate(&self, loader: &Loader) -> RaptorResult<()> {
        match self {
            Self::Program(prog) => {
                let mut exec = PrintExecutor::new();

                exec.run(loader, prog)?;
            }

            Self::DockerSource(image) => {
                info!("Would download docker image [{image}]");
            }
        }

        Ok(())
    }

    fn build(&self, loader: &Loader, layers: &[Utf8PathBuf], layer: LayerInfo) -> RaptorResult<()> {
        match self {
            Self::Program(prog) => {
                let sandbox = Sandbox::new(layers, Utf8Path::new(&layer.work_path()))?;

                let mut exec = Executor::new(sandbox, layer);

                exec.run(loader, prog)?;

                exec.finish()?;
            }

            Self::DockerSource(image) => {
                let work_path = layer.work_path();

                fs::create_dir_all(&work_path)?;

                let dc = DockerDownloader::new(Utf8PathBuf::from("cache"))?;

                let layers = dc.pull(image, "linux", "amd64")?;

                for layer in layers.layers {
                    info!("Extracting layer [{}]", layer.digest);

                    let filename = dc.layer_file_name(&layer.digest);

                    Command::new("sudo")
                        .arg("tar")
                        .arg("-x")
                        .arg("-C")
                        .arg(&work_path)
                        .arg("-f")
                        .arg(filename)
                        .status()?;
                }
            }
        }

        Ok(())
    }
}

impl<'a> RaptorBuilder<'a> {
    pub fn new(loader: Loader<'a>, dry_run: bool) -> Self {
        Self {
            loader,
            dry_run,
            programs: HashMap::new(),
        }
    }

    pub fn load(&mut self, path: impl AsRef<Utf8Path>) -> RaptorResult<Arc<Program>> {
        let program = match self.loader.parse_template(&path, context! {}) {
            Ok(res) => res,
            Err(err) => {
                self.loader.explain_error(&err)?;
                return Err(err);
            }
        };

        let key = path.as_ref().into();

        let res = self
            .programs
            .entry(key)
            .or_insert_with(|| Arc::new(program));
        Ok(res.clone())
    }

    pub fn recurse(
        &mut self,
        program: Arc<Program>,
        visitor: &mut impl FnMut(BuildTarget),
    ) -> RaptorResult<()> {
        match program.from()? {
            Some(FromSource::Docker(mut image)) => {
                if !image.contains('/') {
                    image = format!("library/{image}");
                }
                let source = dregistry::reference::parse(&image)?;
                visitor(BuildTarget::DockerSource(source));
            }
            Some(FromSource::Raptor(from)) => {
                let from = format!("{from}.rapt");
                let base = program.path.try_parent()?;

                let filename = base.join(from);
                let fromprog = self.load(filename)?;

                self.recurse(fromprog, visitor)?;
            }
            None => {}
        }

        visitor(BuildTarget::Program(program));

        Ok(())
    }

    pub fn stack(&mut self, program: Arc<Program>) -> RaptorResult<Vec<BuildTarget>> {
        let mut data: Vec<BuildTarget> = vec![];
        let table = &mut data;

        self.recurse(program, &mut |prog| {
            table.push(prog);
        })?;

        Ok(data)
    }

    pub fn build(&mut self, program: Arc<Program>) -> RaptorResult<()> {
        match self.run_build(program) {
            Ok(res) => Ok(res),
            Err(err) => {
                self.loader.explain_error(&err)?;
                Err(err)
            }
        }
    }

    fn run_build(&mut self, program: Arc<Program>) -> RaptorResult<()> {
        let programs = self.stack(program)?;

        let mut layers: Vec<Utf8PathBuf> = vec![];

        for prog in programs {
            let layer = prog.layer_info()?;

            let layer_name = layer.name().to_string();
            let work_path = layer.work_path();
            let done_path = layer.done_path();

            if fs::exists(layer.done_path())? {
                info!("{} {}", "Completed".bright_white(), layer_name.yellow());
            } else {
                info!(
                    "{} {}: {}",
                    "Building".bright_white(),
                    layer_name.yellow(),
                    layer.work_path().green()
                );

                if self.dry_run {
                    prog.simulate(&self.loader)?;
                } else {
                    prog.build(&self.loader, &layers, layer)?;

                    debug!("Layer {layer_name} finished. Moving {work_path} -> {done_path}");
                    fs::rename(&work_path, &done_path)?;
                }
            }

            layers.push(Utf8PathBuf::from(done_path));
        }

        Ok(())
    }
}
