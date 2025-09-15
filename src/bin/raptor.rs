use std::collections::HashMap;
use std::io::{stdout, IsTerminal};

use camino::{Utf8Path, Utf8PathBuf};
use camino_tempfile::Builder;
use clap::Parser as _;
use colored::Colorize;
use log::{debug, error, info};

use raptor::build::{BuildTargetStats, Presenter, RaptorBuilder};
use raptor::dsl::MountType;
use raptor::program::Loader;
use raptor::sandbox::{ConsoleMode, Sandbox};
use raptor::{RaptorError, RaptorResult};
use uuid::Uuid;

#[derive(clap::Parser, Debug)]
#[command(about, long_about = None, styles=raptor::util::clapcolor::style())]
struct Cli {
    /// Make no changes (print what would have been done)
    #[arg(short = 'n', long, global = true)]
    no_act: bool,

    #[command(subcommand)]
    mode: Mode,
}

#[derive(clap::Subcommand, Clone, Debug)]
#[group(multiple = false)]
#[allow(clippy::struct_excessive_bools)]
enum Mode {
    /// Build mode: generate output from raptor files
    #[command(alias = "b")]
    Build {
        /// Targets to build <target1.rapt target2.rapt ...>
        #[arg(value_name = "targets")]
        targets: Vec<Utf8PathBuf>,
    },

    /// Dump mode: show output from templating pass
    #[command(alias = "d")]
    Dump {
        /// Targets to dump <target1.rapt target2.rapt ...>
        #[arg(value_name = "targets")]
        targets: Vec<Utf8PathBuf>,
    },

    /// Check mode: check validity of input files only
    #[command(alias = "c")]
    Check {
        /// Targets to check <target1.rapt target2.rapt ...>
        #[arg(value_name = "targets")]
        targets: Vec<Utf8PathBuf>,
    },

    /// Run mode: run a shell or command inside the layer
    #[command(alias = "e")]
    #[command(after_help = [
        "  If <state-dir> is specified, any changes made in the session will be saved there.",
        "",
        "  If <state-dir> is not specified, all changes will be lost."
    ].join("\n"))]
    Run {
        /// Target to run
        #[arg(value_name = "target")]
        target: Utf8PathBuf,

        /// State directory for changes (ephemeral if unset)
        #[arg(short = 's', long)]
        #[arg(value_name = "state-dir")]
        state: Option<Utf8PathBuf>,

        #[arg(short = 'M', long, num_args = 2, action = clap::ArgAction::Append)]
        mount: Vec<String>,

        /// Command arguments (defaults to interactive shell if unset)
        #[arg(value_name = "arguments")]
        args: Vec<String>,
    },

    /// Show mode: print list of build targets
    #[command(alias = "s")]
    Show { dirs: Vec<Utf8PathBuf> },
}

#[allow(dead_code)]
impl Mode {
    const fn dump(&self) -> bool {
        matches!(self, Self::Dump { .. })
    }

    const fn build(&self) -> bool {
        matches!(self, Self::Build { .. } | Self::Run { .. })
    }

    const fn run(&self) -> bool {
        matches!(self, Self::Run { .. })
    }

    const fn check(&self) -> bool {
        matches!(self, Self::Check { .. })
    }

    const fn show(&self) -> bool {
        matches!(self, Self::Show { .. })
    }

    fn mounts(&self) -> HashMap<&str, &str> {
        let mut res = HashMap::new();

        let Self::Run { mount, .. } = self else {
            return res;
        };

        for kv in mount.chunks_exact(2) {
            res.insert(&kv[0], &kv[1]);
        }

        res
    }
}

fn check_for_falcon_binary() -> RaptorResult<()> {
    if !std::fs::exists(Sandbox::FALCON_PATH)? {
        error!(
            "The program falcon could not be found\n\n  {}\n",
            Sandbox::FALCON_PATH
        );

        info!("Please compile it before proceeding:");

        eprintln!();
        eprintln!("  {}", "# install packages".dimmed());
        eprintln!("  apt-get update && apt-get install musl-tools -y");

        eprintln!();
        eprintln!("  {}", "# add rust musl target".dimmed());
        eprintln!("  rustup target add x86_64-unknown-linux-musl");

        eprintln!();
        eprintln!("  {}", "# compile falcon".dimmed());
        eprintln!("  cargo build --target x86_64-unknown-linux-musl --release --package falcon --bin=falcon");
        std::process::exit(1);
    }
    Ok(())
}

#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn raptor() -> RaptorResult<()> {
    let args = Cli::parse();

    check_for_falcon_binary()?;

    let loader = Loader::new()?.with_dump(args.mode.dump());

    let mut builder = RaptorBuilder::new(loader, args.no_act);

    let mounts = args.mode.mounts();

    match &args.mode {
        Mode::Dump { ref targets } | Mode::Check { ref targets } | Mode::Build { ref targets } => {
            for file in targets {
                let program = builder.load(file)?;

                if args.mode.build() {
                    builder.build(program)?;
                }
            }

            if args.mode.check() {
                info!("No errors detected.");
            }
        }

        Mode::Run {
            target,
            state,
            args,
            ..
        } => {
            let program = builder.load(target)?;

            builder.build(program.clone())?;

            let uuid = Uuid::new_v4();

            let mut layers = vec![];

            for layer in builder.stack(program.clone())? {
                layers.push(layer.layer_info(&mut builder)?.done_path());
            }

            let tempdir = Builder::new().prefix("raptor-temp-").tempdir()?;

            /* the ephemeral root directory needs to have /usr for systemd-nspawn to accept it */
            let root = tempdir.path().join("root");
            std::fs::create_dir_all(root.join("usr"))?;

            let work = state.clone().unwrap_or_else(|| tempdir.path().join("work"));

            std::fs::create_dir_all(&work)?;

            let console_mode = if stdout().is_terminal() {
                ConsoleMode::Interactive
            } else {
                ConsoleMode::Pipe
            };

            let mut spawn = Sandbox::builder()
                .uuid(uuid)
                .console(console_mode)
                .arg("--background=")
                .arg("--no-pager")
                .root_overlays(&layers)
                .root_overlay(work)
                .directory(&root)
                .args(args);
            /* .setenv("FALCON_LOG_LEVEL", "debug") */

            for mount in program.mounts() {
                let src: Utf8PathBuf = mounts
                    .get(&mount.name.as_str())
                    .ok_or_else(|| RaptorError::MountMissing(mount.clone()))?
                    .into();
                match mount.opts.mtype {
                    MountType::Simple => {
                        spawn = spawn.bind(&src, Utf8Path::new(&mount.dest));
                    }

                    MountType::Layers => {
                        let program = builder.load(src)?;
                        let layers = builder.build(program)?;

                        for layer in &layers {
                            let filename = layer.file_name().unwrap();
                            spawn = spawn.bind_ro(layer, &mount.dest.join(filename));
                        }
                    }

                    MountType::Overlay => {
                        let program = builder.load(src)?;
                        let layers = builder.build(program)?;
                        spawn = spawn.overlay_ro(&layers, &mount.dest);
                    }
                }
            }

            spawn.command().spawn()?.wait()?;
        }

        Mode::Show { dirs } => {
            let mut stats = BuildTargetStats::new();
            for target in dirs {
                let program = builder.load(target)?;
                let stack = builder.stack(program)?;

                stats.merge(stack)?;
            }

            Presenter::new(&stats).present()?;
        }
    }

    Ok(())
}

fn main() {
    colog::init();

    match raptor() {
        Ok(()) => {
            debug!("Raptor completed successfully");
        }

        Err(err) => {
            debug!("Raptor failed: {err}");
            std::process::exit(1);
        }
    }
}
