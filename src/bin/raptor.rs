use camino::Utf8PathBuf;
use clap::Parser as _;
use log::{error, info};

use raptor::build::{Cacher, RaptorBuilder};
use raptor::program::Loader;
use raptor::RaptorResult;

#[derive(clap::Parser, Debug)]
#[command(about, long_about = None)]
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

fn raptor() -> RaptorResult<()> {
    let args = Cli::parse();

    let loader = Loader::new("", args.mode.dump)?;
    let mut builder = RaptorBuilder::new(loader);

    for file in args.input {
        let program = builder.load(file)?;

        let hash = Cacher::cache_key(&program)?;
        info!("Hash: {hash:X}");

        print!("{program}");

        if args.no_act {
            continue;
        }

        builder.exec(&program)?;
    }

    Ok(())
}

fn main() {
    colog::init();
    if let Err(err) = raptor() {
        error!("Error: {err}");
        std::process::exit(1);
    }
}
