use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{Read, stdout};
use std::os::fd::{AsFd, AsRawFd, OwnedFd, RawFd};
use std::time::Duration;

use camino::Utf8PathBuf;
use clap::{ArgAction, CommandFactory, Parser as _};
use clap_complete::Shell;
use colored::Colorize;
use crossbeam::channel::TryRecvError;
use itertools::Itertools;
use log::{LevelFilter, debug, error, info};
use nix::libc;
use nix::poll::{PollFd, PollFlags};
use nix::pty::ForkptyResult;
use nix::sys::wait::waitpid;
use nix::unistd::Uid;
use raptor::util::tty::TtyIoctl;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{DefaultTerminal, restore};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tui_term::vt100;
use tui_term::widget::PseudoTerminal;

use raptor::build::{BuildTargetStats, Presenter, RaptorBuilder};
use raptor::make::maker::Maker;
use raptor::make::parser::MakeTarget;
use raptor::make::planner::{Job, Planner};
use raptor::program::Loader;
use raptor::runner::Runner;
use raptor::sandbox::Sandbox;
use raptor::{RaptorError, RaptorResult};

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
    Run(RunCmd),

    /// Show mode: print list of build targets
    #[command(alias = "s")]
    Show { dirs: Vec<Utf8PathBuf> },

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
    target: Utf8PathBuf,

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

struct Pane {
    file: File,
    job: Job,
    parser: vt100::Parser,
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::too_many_lines)]
fn raptor(terminal: &mut DefaultTerminal) -> RaptorResult<()> {
    let args = Cli::parse();

    log::set_max_level(args.log_level());

    check_for_falcon_binary()?;

    let loader = Loader::new()?.with_dump(args.mode.dump());

    for [name, path] in args.link.as_chunks().0 {
        loader.add_package(name.into(), path.into());
    }

    let builder = RaptorBuilder::new(loader, args.no_act);

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

        Mode::Make { file, targets } => {
            let maker = Maker::load(file)?;

            maker.add_links(builder.loader());

            let mut plan = Planner::new(&maker, &builder);

            for target in targets {
                plan.add(target)?;
            }

            let (plan, targetlist) = plan.into_plan();

            let (tx, rx) = crossbeam::channel::unbounded::<(OwnedFd, Job)>();

            std::thread::scope(|s| {
                s.spawn(move || -> RaptorResult<()> {
                    let mut panes: HashMap<RawFd, Pane> = HashMap::new();
                    let mut need_resize = false;

                    loop {
                        terminal.try_draw(|f| -> Result<(), std::io::Error> {
                            let chunks = Layout::default()
                                .direction(Direction::Vertical)
                                .margin(1)
                                .constraints(
                                    [Constraint::Percentage(100), Constraint::Min(4)].as_ref(),
                                )
                                .split(f.area());

                            let pane_width = if panes.is_empty() {
                                chunks[0].width
                            } else {
                                (chunks[0].width.saturating_sub(1)) / panes.len() as u16
                            };

                            if need_resize {
                                for pane in panes.values_mut() {
                                    let rows = chunks[0].height;
                                    let cols = pane_width;
                                    pane.parser.set_size(rows, cols);
                                    pane.file.tty_set_size(rows, cols)?;
                                }

                                need_resize = false;
                            }

                            for (index, pane) in panes.values_mut().enumerate() {
                                let block = Block::default()
                                    .borders(Borders::ALL)
                                    .title(format!("{:?}", &pane.job))
                                    .title_alignment(Alignment::Center)
                                    .style(
                                        Style::new().add_modifier(Modifier::BOLD).bg(Color::Blue),
                                    );

                                let screen = pane.parser.screen();
                                let pseudo_term = PseudoTerminal::new(screen).block(block);

                                let pane_chunk = Rect {
                                    x: chunks[0].x + (index as u16 * pane_width),
                                    y: chunks[0].y,
                                    width: pane_width,
                                    height: chunks[0].height,
                                };

                                f.render_widget(pseudo_term, pane_chunk);
                            }

                            let explanation = "Ctrl+q to quit";
                            let explanation = Paragraph::new(explanation)
                                .style(Style::default().add_modifier(Modifier::BOLD))
                                .alignment(Alignment::Center);
                            f.render_widget(explanation, chunks[1]);

                            Ok(())
                        })?;

                        let fds = {
                            let mut pollfds = vec![];
                            for pane in panes.values() {
                                pollfds.push(PollFd::new(pane.file.as_fd(), PollFlags::POLLIN));
                            }

                            nix::poll::poll(&mut pollfds, 100u16)?;

                            pollfds
                                .iter()
                                .filter(|fd| fd.any().unwrap_or(false))
                                .map(AsFd::as_fd)
                                .map(|fd| fd.as_raw_fd())
                                .collect_vec()
                        };

                        let mut buf = [0u8; 1024 * 8];

                        for fd in fds {
                            let sz = panes.get_mut(&fd).unwrap().file.read(&mut buf);
                            match sz {
                                Ok(0) | Err(_) => {
                                    panes.remove(&fd);
                                    need_resize = true;
                                }
                                Ok(sz) => {
                                    panes.get_mut(&fd).unwrap().parser.process(&buf[..sz]);
                                }
                            }
                        }

                        match rx.try_recv() {
                            Ok((fd, job)) => {
                                let raw_fd = fd.as_raw_fd();

                                let pane = Pane {
                                    file: fd.into(),
                                    job,
                                    parser: vt100::Parser::new(25, 80, 0),
                                };
                                panes.insert(raw_fd, pane);
                                need_resize = true;
                            }
                            Err(TryRecvError::Empty) => {}
                            Err(TryRecvError::Disconnected) => {
                                if panes.is_empty() {
                                    return Ok(());
                                }
                            }
                        }
                    }
                });

                s.spawn(|| -> RaptorResult<()> {
                    plan.into_par_iter().try_for_each(|id| {
                        let target = &targetlist[&id];

                        match unsafe { nix::pty::forkpty(None, None)? } {
                            ForkptyResult::Parent { child, master } => {
                                tx.send((master, target.clone())).expect("tx");
                                waitpid(child, None)?;
                            }
                            ForkptyResult::Child => {
                                match target {
                                    Job::Build(build) => {
                                        builder.build_layer(
                                            &build.layers,
                                            &build.target,
                                            &build.layerinfo,
                                        )?;
                                    }
                                    Job::Run(run_target) => {
                                        maker.run_job(&builder, run_target)?;
                                    }
                                }

                                std::thread::sleep(Duration::from_millis(100));

                                unsafe { libc::_exit(0) };
                            }
                        }

                        Ok::<(), RaptorError>(())
                    })?;

                    drop(tx);

                    Ok(())
                });
            });
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

    let mut terminal = ratatui::init();
    let res = raptor(&mut terminal);
    restore();

    match res {
        Ok(()) => {
            debug!("Raptor completed successfully");
        }

        Err(err) => {
            error!("Raptor failed: {err}");
            std::process::exit(1);
        }
    }
}
