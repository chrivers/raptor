use std::fs;
use std::hash::{Hash, Hasher};
use std::process::Command;
use std::sync::Arc;

use camino::{Utf8Path, Utf8PathBuf};
use colored::Colorize;
use dregistry::downloader::DockerDownloader;
use dregistry::source::DockerSource;
use siphasher::sip::SipHasher13;

use crate::RaptorResult;
use crate::build::{Cacher, LayerInfo};
use crate::dsl::Program;
use crate::program::{Executor, Loader, PrintExecutor};
use crate::sandbox::Sandbox;
use raptor_parser::ast::{FromSource, Origin};
use raptor_parser::util::module_name::ModuleName;

pub struct RaptorBuilder<'a> {
    loader: Loader<'a>,
    falcon_path: Utf8PathBuf,
    dry_run: bool,
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
    pub const fn new(loader: Loader<'a>, falcon_path: Utf8PathBuf, dry_run: bool) -> Self {
        Self {
            loader,
            falcon_path,
            dry_run,
        }
    }

    pub fn load(&self, name: &ModuleName) -> RaptorResult<Arc<Program>> {
        let origin = Origin::inline();
        self.loader.load_program(name, origin)
    }

    pub const fn loader<'b>(&'b self) -> &'b Loader<'a> {
        &self.loader
    }

    pub const fn loader_mut<'b>(&'b mut self) -> &'b mut Loader<'a> {
        &mut self.loader
    }

    pub fn layer_info(&self, target: &BuildTarget) -> RaptorResult<LayerInfo> {
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
    }

    pub fn stack(&self, program: Arc<Program>) -> RaptorResult<Vec<BuildTarget>> {
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
                    let fromprog = self.loader.load_program(from, origin.clone())?;

                    next = Some(fromprog);
                }
            }
        }

        data.reverse();

        Ok(data)
    }

    fn simulate(target: &BuildTarget) -> RaptorResult<()> {
        match target {
            BuildTarget::Program(prog) => PrintExecutor::new().run(prog)?,
            BuildTarget::DockerSource(image) => info!("Would download docker image [{image}]"),
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
                let sandbox = Sandbox::new(layers, rootdir, &self.falcon_path)?;

                let mut exec = Executor::new(sandbox);

                exec.run(&self.loader, prog)?;

                exec.finish()?;
            }

            BuildTarget::DockerSource(image) => {
                fs::create_dir_all(rootdir)?;

                let dc = DockerDownloader::new(Utf8PathBuf::from("cache"))?;

                let layers = dc.pull(image, "linux", "amd64")?;

                for layer in layers {
                    info!("Extracting layer [{layer}]");

                    let filename = dc.layer_file_name(&layer);

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
                Self::simulate(prog)?;
            } else {
                self.build(prog, layers, &layer.work_path())?;

                debug!("Layer {layer_name} finished. Moving {work_path} -> {done_path}");
                fs::rename(&work_path, &done_path)?;
            }
        }

        Ok(done_path)
    }

    pub fn build_program(&self, program: Arc<Program>) -> RaptorResult<Vec<Utf8PathBuf>> {
        let programs = self.stack(program)?;

        let mut layers: Vec<Utf8PathBuf> = vec![];

        for prog in &programs {
            let layer_info = self.layer_info(prog)?;
            let done_path = self.build_layer(&layers, prog, &layer_info)?;
            layers.push(done_path);
        }

        Ok(layers)
    }
}
