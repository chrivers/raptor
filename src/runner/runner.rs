use std::collections::HashMap;
use std::fs::{self, File};
use std::hash::BuildHasher;
use std::io::{ErrorKind, IsTerminal, stdout};
use std::process::ExitStatus;

use camino::{Utf8Path, Utf8PathBuf};
use camino_tempfile::{Builder, Utf8TempDir};
use raptor_parser::util::module_name::ModuleName;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::build::RaptorBuilder;
use crate::dsl::Program;
use crate::sandbox::{BindMount, ConsoleMode, Sandbox, SpawnBuilder};
use crate::{RaptorError, RaptorResult};
use raptor_parser::ast::{InstMount, MountType};

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
        prog_mounts: &[&InstMount],
        builder: &RaptorBuilder,
        mounts: &HashMap<String, Vec<String>, S>,
        tempdir: impl AsRef<Utf8Path>,
    ) -> RaptorResult<Self>;
}

impl AddMounts for SpawnBuilder {
    fn add_mounts<S: BuildHasher>(
        mut self,
        prog_mounts: &[&InstMount],
        builder: &RaptorBuilder,
        mounts: &HashMap<String, Vec<String>, S>,
        tempdir: impl AsRef<Utf8Path>,
    ) -> RaptorResult<Self> {
        for mount in prog_mounts {
            let mount_list = mounts.get(&mount.name);

            if mount.opts.optional && mount_list.is_none() {
                continue;
            }

            let srcs: Vec<String> = mount_list
                .ok_or_else(|| RaptorError::MountMissing((*mount).clone()))?
                .iter()
                .map(ToString::to_string)
                .collect();

            match mount.opts.mtype {
                MountType::File => {
                    if srcs.len() != 1 {
                        return Err(RaptorError::SingleMountOnly(mount.opts.mtype));
                    }

                    File::options().create(true).append(true).open(&srcs[0])?;

                    let bind = BindMount::new(&srcs[0], Utf8Path::new(&mount.dest));

                    self = if mount.opts.readonly {
                        self.bind_ro(bind)
                    } else {
                        self.bind(bind)
                    };
                }

                MountType::Simple => {
                    if srcs.len() != 1 {
                        return Err(RaptorError::SingleMountOnly(mount.opts.mtype));
                    }

                    match fs::create_dir(&srcs[0]) {
                        Ok(()) => {}
                        Err(err) if err.kind() == ErrorKind::AlreadyExists => {}
                        Err(err) => {
                            error!("Failed to create mount directory {:?}: {err}", &srcs[0]);
                            Err(err)?;
                        }
                    }

                    let bind = BindMount::new(&srcs[0], Utf8Path::new(&mount.dest));

                    self = if mount.opts.readonly {
                        self.bind_ro(bind)
                    } else {
                        self.bind(bind)
                    };
                }

                MountType::Layers => {
                    let mut info = MountsInfo::new();

                    for src in srcs {
                        let name = ModuleName::from(&src);
                        let program = builder.load(&name)?;
                        let layers = builder.build_program(program)?;

                        info.targets.push(src.clone());

                        let layer_info = info.layers.entry(src).or_default();

                        for layer in &layers {
                            let filename = layer.file_name().unwrap();
                            layer_info.push(filename.to_string());
                            self = self.bind_ro(BindMount::new(layer, mount.dest.join(filename)));
                        }
                    }

                    let listfile = tempdir.as_ref().join(format!("mounts-{}", mount.name));
                    fs::write(&listfile, serde_json::to_string_pretty(&info)? + "\n")?;

                    self = self.bind_ro(BindMount::new(&listfile, mount.dest.join("raptor.json")));
                }

                MountType::Overlay => {
                    if srcs.len() != 1 {
                        return Err(RaptorError::SingleMountOnly(mount.opts.mtype));
                    }

                    let program = builder.load(&ModuleName::from(&srcs[0]))?;
                    let layers = builder.build_program(program)?;
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

const EMPTY: &[String] = &[];

pub struct Runner<'a> {
    tempdir: Utf8TempDir,
    env: &'a [String],
    args: &'a [String],
    entrypoint: &'a [String],
    state_dir: Option<Utf8PathBuf>,
    mounts: HashMap<String, Vec<String>>,
}

impl<'a> Runner<'a> {
    pub fn new() -> RaptorResult<Self> {
        let tempdir = Builder::new().prefix("raptor-temp-").tempdir()?;
        Ok(Self {
            tempdir,
            env: EMPTY,
            args: EMPTY,
            entrypoint: EMPTY,
            state_dir: None,
            mounts: HashMap::new(),
        })
    }

    pub fn with_state_dir(&mut self, dir: Utf8PathBuf) -> &mut Self {
        self.state_dir = Some(dir);
        self
    }

    pub const fn with_args(&mut self, args: &'a [String]) -> &mut Self {
        self.args = args;
        self
    }

    pub fn with_mounts(&mut self, mounts: HashMap<String, Vec<String>>) -> &mut Self {
        self.mounts = mounts;
        self
    }

    pub fn add_mount(&mut self, name: &str, mount: String) -> &mut Self {
        if let Some(map) = self.mounts.get_mut(name) {
            map.push(mount);
        } else {
            self.mounts.insert(name.to_string(), vec![mount]);
        }

        self
    }

    pub const fn with_entrypoint(&mut self, entrypoint: &'a [String]) -> &mut Self {
        self.entrypoint = entrypoint;
        self
    }

    pub const fn with_env(&mut self, env: &'a [String]) -> &mut Self {
        self.env = env;
        self
    }

    pub fn spawn(
        self,
        program: &Program,
        builder: &RaptorBuilder,
        layers: &[Utf8PathBuf],
    ) -> RaptorResult<ExitStatus> {
        /* the ephemeral root directory needs to have /usr for systemd-nspawn to accept it */
        let root = self.tempdir.path().join("root");
        fs::create_dir_all(root.join("usr"))?;

        let work = self
            .state_dir
            .unwrap_or_else(|| self.tempdir.path().join("work"));

        fs::create_dir_all(&work)?;

        let mut command = vec![];

        if !self.entrypoint.is_empty() {
            command.extend(self.entrypoint.iter().map(String::as_str));
        } else if let Some(entr) = program.entrypoint() {
            command.extend(entr.entrypoint.iter().map(String::as_str));
        } else {
            command.extend(["/bin/sh", "-c"]);
        }

        if !self.args.is_empty() {
            command.extend(self.args.iter().map(String::as_str));
        } else if let Some(cmd) = program.cmd() {
            command.extend(cmd.cmd.iter().map(String::as_str));
        } else {
            return Err(RaptorError::NoCommandSpecified);
        }

        trace!("Command {command:?}");

        let console_mode = if stdout().is_terminal() {
            ConsoleMode::Interactive
        } else {
            ConsoleMode::Pipe
        };

        let res = Sandbox::builder()
            .uuid(Uuid::new_v4())
            .console(console_mode)
            .arg("--background=")
            .arg("--no-pager")
            .root_overlays(layers)
            .root_overlay(work)
            .directory(&root)
            .bind(BindMount::new("/dev/kvm", "/dev/kvm"))
            .args(&command)
            .add_mounts(&program.mounts(), builder, &self.mounts, &self.tempdir)?
            .add_environment(self.env)
            .command()
            .spawn()?
            .wait()?;

        Ok(res)
    }
}
