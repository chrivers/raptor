use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::process::Command;
use std::sync::Arc;

use camino::{Utf8Path, Utf8PathBuf};
use colored::Colorize;
use dregistry::downloader::DockerDownloader;
use dregistry::source::DockerSource;
use minijinja::context;
use siphasher::sip::SipHasher13;

use crate::RaptorResult;
use crate::build::{Cacher, LayerInfo};
use crate::dsl::Program;
use crate::program::{Executor, Loader, PrintExecutor};
use crate::sandbox::Sandbox;
use raptor_parser::ast::{FromSource, Origin};

pub struct RaptorBuilder<'a> {
    loader: Loader<'a>,
    dry_run: bool,
    programs: HashMap<Utf8PathBuf, Arc<Program>>,
}

#[derive(Debug, Clone)]
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

impl<'a> RaptorBuilder<'a> {
    pub fn new(loader: Loader<'a>, dry_run: bool) -> Self {
        Self {
            loader,
            dry_run,
            programs: HashMap::new(),
        }
    }

    pub fn load(&mut self, path: impl AsRef<Utf8Path>) -> RaptorResult<Arc<Program>> {
        let key = path.as_ref();

        if let Some(program) = self.programs.get(key) {
            return Ok(program.clone());
        }

        let program = match self.loader.parse_template(&path, context! {}) {
            Ok(res) => res,
            Err(err) => {
                self.loader.explain_error(&err)?;
                return Err(err);
            }
        };

        self.programs.insert(key.into(), Arc::new(program));

        Ok(self.programs[key].clone())
    }

    pub const fn loader<'b>(&'b mut self) -> &'b mut Loader<'a> {
        &mut self.loader
    }

    pub fn load_with_source(
        &mut self,
        path: impl AsRef<Utf8Path>,
        source: Origin,
    ) -> RaptorResult<Arc<Program>> {
        self.loader.push_origin(source);
        let res = self.load(path);
        self.loader.pop_origin();
        res
    }

    pub fn layer_info(&mut self, target: &BuildTarget) -> RaptorResult<LayerInfo> {
        let name;
        let hash;

        match target {
            BuildTarget::Program(prog) => {
                debug!("Calculating hash for layer {}", prog.path);

                name = prog.path.file_stem().unwrap().into();
                hash = Cacher::cache_key(prog, self)?;
            }

            BuildTarget::DockerSource(image) => {
                debug!("Calculating hash for image {image}");

                name = image.safe_file_name()?;

                let mut state = SipHasher13::new();
                image.hash(&mut state);
                hash = state.finish();
            }
        }

        Ok(LayerInfo::new(name.to_string(), hash))
    }

    pub fn clear_cache(&mut self) {
        self.loader.clear_cache();
        self.programs.clear();
    }

    pub fn stack(&mut self, program: Arc<Program>) -> RaptorResult<Vec<BuildTarget>> {
        let mut data: Vec<BuildTarget> = vec![];

        let mut next = Some(program);

        while let Some(prog) = next.take() {
            data.push(BuildTarget::Program(prog.clone()));

            let Some((source, origin)) = prog.from() else {
                continue;
            };

            match source {
                FromSource::Docker(src) => {
                    let image = if src.contains('/') {
                        src.clone()
                    } else {
                        format!("library/{src}")
                    };
                    let source = dregistry::reference::parse(&image)?;
                    data.push(BuildTarget::DockerSource(source));
                }

                FromSource::Raptor(from) => {
                    let fromprog = self
                        .loader
                        .to_program_path(from, origin)
                        .and_then(|path| self.load_with_source(path, origin.clone()))?;

                    next = Some(fromprog);
                }
            }
        }

        Ok(data)
    }

    pub fn build_program(&mut self, program: Arc<Program>) -> RaptorResult<Vec<Utf8PathBuf>> {
        match self.run_build(program) {
            Ok(res) => Ok(res),
            Err(err) => {
                self.loader.explain_error(&err)?;
                Err(err)
            }
        }
    }

    fn simulate(&self, target: &BuildTarget) -> RaptorResult<()> {
        match target {
            BuildTarget::Program(prog) => {
                let mut exec = PrintExecutor::new();

                exec.run(&self.loader, prog)?;
            }

            BuildTarget::DockerSource(image) => {
                info!("Would download docker image [{image}]");
            }
        }

        Ok(())
    }

    fn build(
        &self,
        target: &BuildTarget,
        layers: &[Utf8PathBuf],
        rootdir: &Utf8Path,
    ) -> RaptorResult<()> {
        match target {
            BuildTarget::Program(prog) => {
                let sandbox = Sandbox::new(layers, Utf8Path::new(&rootdir))?;

                let mut exec = Executor::new(sandbox);

                exec.run(&self.loader, prog)?;

                exec.finish()?;
            }

            BuildTarget::DockerSource(image) => {
                fs::create_dir_all(rootdir)?;

                let dc = DockerDownloader::new(Utf8PathBuf::from("cache"))?;

                let layers = dc.pull(image, "linux", "amd64")?;

                for layer in layers.layers {
                    info!("Extracting layer [{}]", layer.digest);

                    let filename = dc.layer_file_name(&layer.digest);

                    Command::new("tar")
                        .arg("-x")
                        .arg("-C")
                        .arg(rootdir)
                        .arg("-f")
                        .arg(filename)
                        .status()?;
                }
            }
        }

        Ok(())
    }

    pub fn build_layer(
        &self,
        layers: &[Utf8PathBuf],
        prog: &BuildTarget,
        layer: &LayerInfo,
    ) -> RaptorResult<Utf8PathBuf> {
        let layer_name = layer.name().to_string();
        let work_path = layer.work_path();
        let done_path = layer.done_path();

        if fs::exists(layer.done_path())? {
            info!(
                "{} [{}] {}",
                "Completed".bright_white(),
                layer.hash().dimmed(),
                layer_name.yellow()
            );
        } else {
            info!(
                "{} {}: {}",
                "Building".bright_white(),
                layer_name.yellow(),
                layer.work_path().as_str().green()
            );

            if self.dry_run {
                self.simulate(prog)?;
            } else {
                self.build(prog, layers, &layer.work_path())?;

                debug!("Layer {layer_name} finished. Moving {work_path} -> {done_path}");
                fs::rename(&work_path, &done_path)?;
            }
        }

        Ok(done_path)
    }

    fn run_build(&mut self, program: Arc<Program>) -> RaptorResult<Vec<Utf8PathBuf>> {
        let programs = self.stack(program)?;

        let mut layers: Vec<Utf8PathBuf> = vec![];

        for prog in programs {
            let layer_info = self.layer_info(&prog)?;
            let done_path = self.build_layer(&layers, &prog, &layer_info)?;
            layers.push(done_path);
        }

        Ok(layers)
    }
}
