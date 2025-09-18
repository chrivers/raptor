use std::collections::HashMap;
use std::fs;
use std::hash::BuildHasher;

use camino::{Utf8Path, Utf8PathBuf};

use crate::build::RaptorBuilder;
use crate::dsl::Program;
use crate::sandbox::{BindMount, SpawnBuilder};
use crate::{RaptorError, RaptorResult};
use raptor_parser::ast::MountType;

pub trait AddMounts: Sized {
    fn add_mounts<S: BuildHasher>(
        self,
        program: &Program,
        builder: &mut RaptorBuilder,
        mounts: &HashMap<&str, &str, S>,
        tempdir: &Utf8Path,
    ) -> RaptorResult<Self>;
}

impl AddMounts for SpawnBuilder {
    fn add_mounts<S: BuildHasher>(
        mut self,
        program: &Program,
        builder: &mut RaptorBuilder,
        mounts: &HashMap<&str, &str, S>,
        tempdir: &Utf8Path,
    ) -> RaptorResult<Self> {
        for mount in program.mounts() {
            let src: Utf8PathBuf = mounts
                .get(&mount.name.as_str())
                .ok_or_else(|| RaptorError::MountMissing(mount.clone()))?
                .into();

            match mount.opts.mtype {
                MountType::Simple => {
                    self = self.bind(BindMount::new(&src, Utf8Path::new(&mount.dest)));
                }

                MountType::Layers => {
                    let program = builder.load(src)?;
                    let layers = builder.build(program)?;

                    let mut names = vec![];

                    for layer in &layers {
                        let filename = layer.file_name().unwrap();
                        names.push(filename);
                        self = self.bind_ro(BindMount::new(layer, mount.dest.join(filename)));
                    }
                    names.push("");

                    let listfile = tempdir.join(format!("mounts-{}", mount.name));
                    fs::write(&listfile, names.join("\n"))?;

                    self = self.bind_ro(BindMount::new(&listfile, mount.dest.join("ORDER")));
                }

                MountType::Overlay => {
                    let program = builder.load(src)?;
                    let layers = builder.build(program)?;
                    self = self.overlay_ro(&layers, &mount.dest);
                }
            }
        }

        Ok(self)
    }
}
