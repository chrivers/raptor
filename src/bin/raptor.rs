use std::collections::HashMap;
use std::io::{IsTerminal, stdout};

use camino::Utf8PathBuf;
use camino_tempfile::Builder;
use clap::Parser as _;
use colored::Colorize;
use log::{debug, error, info};

use nix::unistd::Uid;
use raptor::build::{BuildTargetStats, Presenter, RaptorBuilder};
use raptor::program::Loader;
use raptor::runner::AddMounts;
use raptor::sandbox::{BindMount, ConsoleMode, Sandbox};
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
    #[command(alias = "r")]
    #[command(after_help = [
        "  If <state-dir> is specified, any changes made in the session will be saved there.",
        "",
        "  If <state-dir> is not specified, all changes will be lost."
    ].join("\n"))]
    Run(RunCmd),

    /// Show mode: print list of build targets
    #[command(alias = "s")]
    Show { dirs: Vec<Utf8PathBuf> },
}

#[derive(clap::Args, Clone, Debug)]
struct RunCmd {
    /// Target to run
    #[arg(value_name = "target")]
    target: Utf8PathBuf,

    /// State directory for changes (ephemeral if unset)
    #[arg(short = 's', long)]
    #[arg(value_name = "state-dir")]
    state: Option<Utf8PathBuf>,

    /// Environment variables
    #[arg(short = 'e', long)]
    #[arg(value_name = "env")]
    env: Vec<String>,

    /// Specify mounts
    #[arg(short = 'M', long, value_names = ["name", "mount"], num_args = 2, action = clap::ArgAction::Append)]
    mount: Vec<String>,

    #[arg(
        short = 'C',
        long,
        value_name = "mount",
        help = "Specify cache mount. Shorthand for -M cache <mount>"
    )]
    cache: Vec<String>,

    #[arg(
        short = 'I',
        long,
        value_name = "mount",
        help = "Specify input mount. Shorthand for -M input <mount>"
    )]
    input: Vec<String>,

    #[arg(
        short = 'O',
        long,
        value_name = "mount",
        help = "Specify output mount. Shorthand for -M output <mount>"
    )]
    output: Option<String>,

    /// Command arguments (defaults to interactive shell if unset)
    #[arg(value_name = "arguments")]
    args: Vec<String>,
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
}

impl RunCmd {
    fn mounts(&self) -> HashMap<&str, Vec<&str>> {
        let mut res: HashMap<&str, Vec<&str>> = HashMap::new();

        for kv in self.mount.chunks_exact(2) {
            res.entry(&kv[0]).or_default().push(&kv[1]);
        }

        for cache in &self.cache {
            res.entry("cache").or_default().push(cache);
        }

        for input in &self.input {
            res.entry("input").or_default().push(input);
        }

        if let Some(output) = &self.output {
            res.entry("output").or_default().push(output);
        }

        res
    }
}

fn check_for_root() -> RaptorResult<()> {
    if Uid::effective().is_root() {
        Ok(())
    } else {
        error!("Root is required to run!\n\nTry with sudo :)\n");
        Err(RaptorError::RootRequired)
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
        eprintln!(
            "  cargo build --target x86_64-unknown-linux-musl --release --package falcon --bin=falcon"
        );
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

    match &args.mode {
        Mode::Dump { targets } | Mode::Check { targets } | Mode::Build { targets } => {
            for file in targets {
                let program = builder.load(file)?;

                if args.mode.build() {
                    if !args.no_act {
                        check_for_root()?;
                    }
                    builder.build(program)?;
                }
            }

            if args.mode.check() {
                info!("No errors detected.");
            }
        }

        Mode::Run(run) => {
            check_for_root()?;

            let program = builder.load(&run.target)?;

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

            let work = run
                .state
                .clone()
                .unwrap_or_else(|| tempdir.path().join("work"));

            std::fs::create_dir_all(&work)?;

            let mut command = vec![];

            if let Some(entr) = program.entrypoint() {
                command.extend(entr.entrypoint.iter().map(String::as_str));
            } else {
                command.push("/bin/sh");
            }

            if !run.args.is_empty() {
                command.extend(run.args.iter().map(String::as_str));
            } else if let Some(cmd) = program.cmd() {
                command.extend(cmd.cmd.iter().map(String::as_str));
            }

            let console_mode = if stdout().is_terminal() {
                ConsoleMode::Interactive
            } else {
                ConsoleMode::Pipe
            };

            let mut sandbox = Sandbox::builder()
                .uuid(uuid)
                .console(console_mode)
                .arg("--background=")
                .arg("--no-pager")
                .root_overlays(&layers)
                .root_overlay(work)
                .directory(&root)
                .bind(BindMount::new("/dev/kvm", "/dev/kvm"))
                .args(&command)
                .add_mounts(&program, &mut builder, &run.mounts(), tempdir.path())?;

            for env in &run.env {
                if let Some((key, value)) = env.split_once('=') {
                    sandbox = sandbox.setenv(key, value);
                } else {
                    sandbox = sandbox.setenv(env, "");
                }
            }

            let res = sandbox.command().spawn()?.wait()?;

            if !res.success() {
                error!("Run failed with status {}", res.code().unwrap_or_default());
            }
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
