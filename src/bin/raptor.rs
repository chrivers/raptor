use std::collections::HashMap;
use std::env;
use std::io::stdout;

use camino::Utf8PathBuf;
use clap::{ArgAction, CommandFactory, Parser as _};
use clap_complete::Shell;
use colored::Colorize;
use log::{LevelFilter, debug, error, info};
use nix::unistd::Uid;
use raptor::batch::ParallelRunner;
use raptor::tui::TerminalParallelRunner;

use raptor::build::{BuildTargetStats, Presenter, RaptorBuilder};
use raptor::make::maker::Maker;
use raptor::make::parser::MakeTarget;
use raptor::make::planner::Planner;
use raptor::program::Loader;
use raptor::runner::Runner;
use raptor::sandbox::Sandbox;
use raptor::{RaptorError, RaptorResult};
use raptor_parser::util::SafeParent;
use raptor_parser::util::module_name::ModuleName;

#[derive(clap::Parser, Debug)]
#[command(about, long_about = None, styles=raptor::util::clapcolor::style())]
struct Cli {
    /// Make no changes (print what would have been done)
    #[arg(short = 'n', long, global = true)]
    no_act: bool,

    /// Increase verbosity (can be repeated)
    #[arg(short = 'v', long, action = ArgAction::Count, global = true, help_heading="Verbosity")]
    verbose: u8,

    /// Decrease verbosity (can be repeated)
    #[arg(short = 'q', long, action = ArgAction::Count, global = true, help_heading="Verbosity")]
    quiet: u8,

    #[command(subcommand)]
    mode: Mode,

    /// Link raptor packages by path and name
    #[arg(
        short = 'L',
        long,
        value_names = ["name", "path"],
        num_args = 2,
        action = ArgAction::Append,
        global = true,
        help_heading="Link packages",
    )]
    link: Vec<String>,
}

impl Cli {
    #[must_use]
    const fn log_level(&self) -> LevelFilter {
        let verbosity = self.verbose as i32 - self.quiet as i32;
        match verbosity {
            ..=-3 => LevelFilter::Off,
            -2 => LevelFilter::Error,
            -1 => LevelFilter::Warn,
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            2.. => LevelFilter::Trace,
        }
    }
}

#[derive(clap::Subcommand, Clone, Debug)]
#[group(multiple = false)]
#[allow(clippy::struct_excessive_bools)]
enum Mode {
    /// Build mode: generate output from raptor files
    #[command(alias = "b")]
    Build {
        /// Targets to build <target1 target2 ...>
        #[arg(value_name = "targets")]
        targets: Vec<ModuleName>,
    },

    /// Dump mode: show output from templating pass
    #[command(alias = "d")]
    Dump {
        /// Targets to dump <target1 target2 ...>
        #[arg(value_name = "targets")]
        targets: Vec<ModuleName>,
    },

    /// Check mode: check validity of input files only
    #[command(alias = "c")]
    Check {
        /// Targets to check <target1 target2 ...>
        #[arg(value_name = "targets")]
        targets: Vec<ModuleName>,
    },

    /// Run mode: run a shell or command inside the layer
    #[command(alias = "r")]
    Run(RunCmd),

    /// Show mode: print list of build targets
    #[command(alias = "s")]
    Show { dirs: Vec<ModuleName> },

    /// Make mode: run build operations from makefile (Raptor.toml)
    Make {
        #[arg(
            short = 'f',
            long,
            help = "File",
            default_value_t = Utf8PathBuf::from("Raptor.toml")
        )]
        file: Utf8PathBuf,

        targets: Vec<MakeTarget>,

        #[arg(short = 'b', long, help = "Batch mode (disable text user interface)")]
        batch: bool,
    },

    /// Completions mode: generate shell completion scripts
    Completion {
        #[arg(value_name = "shell")]
        shell: Shell,
    },
}

#[derive(clap::Args, Clone, Debug)]
struct RunCmd {
    /// Target to run
    #[arg(value_name = "target")]
    target: ModuleName,

    /// State directory for changes (ephemeral if unset)
    #[arg(short = 's', long)]
    #[arg(value_name = "state-dir")]
    #[arg(help = "The state directory will save the changes made during run")]
    #[arg(long_help = [
        "If <state-dir> is specified, any changes made in the session will be saved there.",
        "",
        "If <state-dir> is not specified, all changes will be lost."
    ].join("\n"))]
    state: Option<Utf8PathBuf>,

    /// Environment variables
    #[arg(short = 'e', long)]
    #[arg(value_name = "env")]
    env: Vec<String>,

    /// Specify mounts
    #[arg(
        short = 'M',
        long,
        value_names = ["name", "mount"],
        num_args = 2,
        action = ArgAction::Append,
        help_heading="Mount options",
    )]
    mount: Vec<String>,

    #[arg(
        short = 'C',
        long,
        value_name = "mount",
        help = "Specify cache mount. Shorthand for -M cache <mount>",
        help_heading = "Mount options"
    )]
    cache: Vec<String>,

    #[arg(
        short = 'I',
        long,
        value_name = "mount",
        help = "Specify input mount. Shorthand for -M input <mount>",
        help_heading = "Mount options"
    )]
    input: Vec<String>,

    #[arg(
        short = 'O',
        long,
        value_name = "mount",
        help = "Specify output mount. Shorthand for -M output <mount>",
        help_heading = "Mount options"
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
    fn mounts(&self) -> HashMap<String, Vec<String>> {
        let mut res: HashMap<String, Vec<String>> = HashMap::new();

        for kv in self.mount.chunks_exact(2) {
            res.entry(kv[0].clone()).or_default().push(kv[1].clone());
        }

        for cache in &self.cache {
            res.entry("cache".into()).or_default().push(cache.clone());
        }

        for input in &self.input {
            res.entry("input".into()).or_default().push(input.clone());
        }

        if let Some(output) = &self.output {
            res.entry("output".into()).or_default().push(output.clone());
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

fn check_for_falcon_binary() -> RaptorResult<Utf8PathBuf> {
    let falcon_path = Sandbox::find_falcon_dev();

    if falcon_path.is_err() {
        error!("The program falcon could not be found\n\n  {falcon_path:?}\n");

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

    falcon_path
}

fn raptor() -> RaptorResult<()> {
    let args = Cli::parse();

    log::set_max_level(args.log_level());

    let falcon_path = check_for_falcon_binary()?;

    let loader = Loader::new()?.with_dump(args.mode.dump());

    for [name, path] in args.link.as_chunks().0 {
        loader.resolver().add_package(name.into(), path.into());
    }

    let mut builder = RaptorBuilder::new(loader, falcon_path, args.no_act);

    match &args.mode {
        Mode::Dump { targets } | Mode::Check { targets } | Mode::Build { targets } => {
            for file in targets {
                let program = builder.load(file)?;

                if args.mode.build() {
                    if !args.no_act {
                        check_for_root()?;
                    }
                    builder.build_program(program)?;
                }
            }

            if args.mode.check() {
                info!("No errors detected.");
            }
        }

        Mode::Run(run) => {
            check_for_root()?;

            let program = builder.load(&run.target)?;

            builder.build_program(program.clone())?;

            let mut layers = vec![];

            for target in builder.stack(program.clone())? {
                layers.push(builder.layer_info(&target)?.done_path());
            }

            let mut runner = Runner::new()?;

            runner
                .with_args(&run.args)
                .with_env(&run.env)
                .with_mounts(run.mounts());

            if let Some(state_dir) = &run.state {
                runner.with_state_dir(state_dir.clone());
            }

            let res = runner.spawn(&program, &builder, &layers)?;

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

        Mode::Make {
            file,
            targets,
            batch,
        } => {
            builder
                .loader_mut()
                .resolver_mut()
                .set_base(file.try_parent()?);
            let maker = Maker::load(&builder, file)?;

            maker.add_links(builder.loader());

            let mut planner = Planner::new(&maker, &builder);

            if targets.is_empty() {
                for target in maker.rules().run.keys() {
                    planner.add_named_run_job(target)?;
                }
            } else {
                for target in targets {
                    planner.add(target)?;
                }
            }

            if *batch {
                let mut brunner = ParallelRunner::new(&maker);
                brunner.execute(planner)?;
            } else {
                let mut terminal = ratatui::init();
                let mut trunner = TerminalParallelRunner::new(&maker, &mut terminal);
                let res = trunner.execute(planner);
                ratatui::restore();
                res?;
            }
        }

        Mode::Completion { shell } => {
            clap_complete::generate(*shell, &mut Cli::command(), "raptor", &mut stdout());
        }
    }

    Ok(())
}

fn main() {
    let mut builder = colog::default_builder();

    builder.filter(None, LevelFilter::Trace);

    if let Ok(rust_log) = env::var("RUST_LOG") {
        builder.parse_filters(&rust_log);
    }
    builder.init();

    match raptor() {
        Ok(()) => {
            debug!("Raptor completed successfully");
        }

        Err(err) => {
            error!("Raptor failed: {err}");
            std::process::exit(1);
        }
    }
}
