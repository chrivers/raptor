use std::io::{stdout, IsTerminal};

use camino::Utf8PathBuf;
use camino_tempfile::Builder;
use clap::Parser as _;
use colored::Colorize;
use log::{debug, error, info};

use raptor::build::{BuildTargetStats, Presenter, RaptorBuilder};
use raptor::program::Loader;
use raptor::sandbox::{
    ConsoleMode, LinkJournal, ResolvConf, Sandbox, Settings, SpawnBuilder, Timezone,
};
use raptor::RaptorResult;
use uuid::Uuid;

#[derive(clap::Parser, Debug)]
#[command(about, long_about = None, styles=raptor::util::clapcolor::style())]
struct Cli {
    /// Make no changes (print what would have been done)
    #[arg(short = 'n', long)]
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

    /// Enter mode: run a shell or command inside the layer
    #[command(alias = "e")]
    #[command(after_help = [
        "  If <state-dir> is specified, any changes made in the session will be saved there.",
        "",
        "  If <state-dir> is not specified, all changes will be lost."
    ].join("\n"))]
    Enter {
        /// Target to enter
        #[arg(value_name = "target")]
        target: Utf8PathBuf,

        /// State directory for changes (ephemeral if unset)
        #[arg(short = 's', long)]
        #[arg(value_name = "state-dir")]
        state: Option<Utf8PathBuf>,

        /// Command arguments (defaults to interactive shell if unset)
        #[arg(value_name = "arguments")]
        args: Vec<String>,
    },

    /// Show mode: print list of build targets
    #[command(alias = "s")]
    Show { dirs: Option<Vec<Utf8PathBuf>> },
}

#[allow(dead_code)]
impl Mode {
    const fn dump(&self) -> bool {
        matches!(self, Self::Dump { .. })
    }

    const fn build(&self) -> bool {
        matches!(self, Self::Build { .. } | Self::Enter { .. })
    }

    const fn enter(&self) -> bool {
        matches!(self, Self::Enter { .. })
    }

    const fn check(&self) -> bool {
        matches!(self, Self::Check { .. })
    }

    const fn show(&self) -> bool {
        matches!(self, Self::Show { .. })
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
        eprintln!("  cargo build --target x86_64-unknown-linux-musl --release --bin=falcon");
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

    match args.mode {
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

        Mode::Enter {
            target,
            state,
            args,
        } => {
            let program = builder.load(target)?;

            builder.build(program.clone())?;

            let uuid = Uuid::new_v4();

            let mut layers = vec![];

            for layer in builder.stack(program)? {
                layers.push(layer.layer_info()?.done_path());
            }

            let tempdir = Builder::new().prefix("raptor-temp-").tempdir()?;

            /* the ephemeral root directory needs to have /usr for systemd-nspawn to accept it */
            let root = tempdir.path().join("root");
            std::fs::create_dir_all(root.join("usr"))?;

            let work = state.unwrap_or_else(|| tempdir.path().join("work"));

            std::fs::create_dir_all(&work)?;

            let console_mode = if stdout().is_terminal() {
                ConsoleMode::Interactive
            } else {
                ConsoleMode::Pipe
            };

            let spawn = SpawnBuilder::new()
                .quiet(true)
                .sudo(true)
                .uuid(uuid)
                .link_journal(LinkJournal::No)
                .resolv_conf(ResolvConf::Off)
                .timezone(Timezone::Off)
                .settings(Settings::False)
                .console(console_mode)
                .arg("--background=")
                .arg("--no-pager")
                .root_overlays(&layers)
                .root_overlay(&work)
                .directory(&root)
                .args(&args);
            /* .setenv("FALCON_LOG_LEVEL", "debug") */

            spawn.command().spawn()?.wait()?;
        }

        Mode::Show { dirs } => {
            let mut stats = BuildTargetStats::new();
            for target in dirs.unwrap() {
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
