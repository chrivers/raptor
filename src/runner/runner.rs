use std::collections::HashMap;
use std::fs::{self, File};
use std::hash::BuildHasher;

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use crate::build::RaptorBuilder;
use crate::dsl::Program;
use crate::sandbox::{BindMount, SpawnBuilder};
use crate::{RaptorError, RaptorResult};
use raptor_parser::ast::MountType;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MountsInfo {
    targets: Vec<String>,
    layers: HashMap<String, Vec<String>>,
}

impl MountsInfo {
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
            layers: HashMap::new(),
        }
    }
}

pub trait AddMounts: Sized {
    fn add_mounts<S: BuildHasher>(
        self,
        program: &Program,
        builder: &mut RaptorBuilder,
        mounts: &HashMap<&str, Vec<&str>, S>,
        tempdir: &Utf8Path,
    ) -> RaptorResult<Self>;
}

impl AddMounts for SpawnBuilder {
    fn add_mounts<S: BuildHasher>(
        mut self,
        program: &Program,
        builder: &mut RaptorBuilder,
        mounts: &HashMap<&str, Vec<&str>, S>,
        tempdir: &Utf8Path,
    ) -> RaptorResult<Self> {
        for mount in program.mounts() {
            let srcs: Vec<Utf8PathBuf> = mounts
                .get(&mount.name.as_str())
                .ok_or_else(|| RaptorError::MountMissing(mount.clone()))?
                .iter()
                .map(Into::into)
                .collect();

            match mount.opts.mtype {
                MountType::File => {
                    if srcs.len() != 1 {
                        return Err(RaptorError::SingleMountOnly(mount.opts.mtype));
                    }

                    File::options().create(true).append(true).open(&srcs[0])?;

                    self = self.bind(BindMount::new(&srcs[0], Utf8Path::new(&mount.dest)));
                }

                MountType::Simple => {
                    if srcs.len() != 1 {
                        return Err(RaptorError::SingleMountOnly(mount.opts.mtype));
                    }

                    self = self.bind(BindMount::new(&srcs[0], Utf8Path::new(&mount.dest)));
                }

                MountType::Layers => {
                    let mut info = MountsInfo::new();

                    for src in srcs {
                        let program = builder.load(&src)?;
                        let name = program.path.with_extension("").as_str().replace('/', ".");

                        let layers = builder.build(program)?;

                        info.targets.push(name.clone());

                        let layer_info = info.layers.entry(name).or_default();

                        for layer in &layers {
                            let filename = layer.file_name().unwrap();
                            layer_info.push(filename.to_string());
                            self = self.bind_ro(BindMount::new(layer, mount.dest.join(filename)));
                        }
                    }

                    let listfile = tempdir.join(format!("mounts-{}", mount.name));
                    fs::write(&listfile, serde_json::to_string_pretty(&info)? + "\n")?;

                    self = self.bind_ro(BindMount::new(&listfile, mount.dest.join("raptor.json")));
                }

                MountType::Overlay => {
                    if srcs.len() != 1 {
                        return Err(RaptorError::SingleMountOnly(mount.opts.mtype));
                    }

                    let program = builder.load(&srcs[0])?;
                    let layers = builder.build(program)?;
                    self = self.overlay_ro(&layers, &mount.dest);
                }
            }
        }

        Ok(self)
    }
}

pub trait AddEnvironment: Sized {
    #[must_use]
    fn add_environment(self, env: &[String]) -> Self;
}

impl AddEnvironment for SpawnBuilder {
    fn add_environment(mut self, envs: &[String]) -> Self {
        for env in envs {
            if let Some((key, value)) = env.split_once('=') {
                self = self.setenv(key, value);
            } else {
                self = self.setenv(env, "");
            }
        }

        self
    }
}
