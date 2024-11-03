use camino::Utf8PathBuf;
use clap::Parser as _;
use log::info;
use pest::Parser;
use raptor::parser::raptorfile::{RaptorFileParser, Rule};

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
struct Mode {
    /// Build mode: generate output from raptor files
    #[arg(short = 'B', long)]
    build: bool,

    /// Check mode: check validity of input files only
    #[arg(short = 'C', long)]
    check: bool,

    /// Show mode: print list of build targets
    #[arg(short = 'S', long)]
    show: bool,
}

#[derive(thiserror::Error, Debug)]
enum RaptorError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    PestError(#[from] pest::error::Error<Rule>),
}

type Result<T> = std::result::Result<T, RaptorError>;

fn main() -> Result<()> {
    colog::init();
    info!("Raptor");

    let args = Cli::parse();

    for file in args.input {
        let source = std::fs::read_to_string(file)?;
        let mut ast = RaptorFileParser::parse(Rule::FILE, &source)?;
        for node in ast.next().unwrap().into_inner() {
            println!("ast: {node}");
            println!("--------------------------------------------------------------------------------");
        }
    }

    Ok(())
}
