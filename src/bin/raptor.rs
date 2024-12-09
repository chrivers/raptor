use camino::Utf8PathBuf;
use clap::Parser as _;
use colored::Colorize;
use log::{debug, error, info};

use raptor::build::RaptorBuilder;
use raptor::program::Loader;
use raptor::sandbox::Sandbox;
use raptor::RaptorResult;

#[derive(clap::Parser, Debug)]
#[command(about, long_about = None, styles=raptor::util::clapcolor::style())]
struct Cli {
    /// Make no changes (print what would have been done)
    #[arg(short = 'n', long)]
    no_act: bool,

    #[command(flatten)]
    mode: Mode,

    /// Input files
    input: Vec<Utf8PathBuf>,
}

#[derive(clap::Args, Clone, Debug)]
#[group(multiple = false)]
#[allow(clippy::struct_excessive_bools)]
struct Mode {
    /// Build mode: generate output from raptor files
    #[arg(short = 'B', long)]
    build: bool,

    /// Dump mode: show output from templating pass
    #[arg(short = 'D', long)]
    dump: bool,

    /// Check mode: check validity of input files only
    #[arg(short = 'C', long)]
    check: bool,

    /// Show mode: print list of build targets
    #[arg(short = 'S', long)]
    show: bool,
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

fn raptor() -> RaptorResult<()> {
    let args = Cli::parse();

    check_for_falcon_binary()?;

    let loader = Loader::new("", args.mode.dump)?;
    let mut builder = RaptorBuilder::new(loader, args.no_act);

    for file in args.input {
        let program = builder.load(file)?;

        builder.build(program)?;
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
