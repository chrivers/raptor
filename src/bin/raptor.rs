use camino::Utf8PathBuf;
use clap::Parser as _;
use log::error;

use minijinja::context;
use raptor::program::{Executor, Loader};
use raptor::sandbox::Sandbox;
use raptor::{template, RaptorResult};

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

    let root_context = context!();

    for file in args.input {
        let mut loader = Loader::new(template::make_environment()?, args.mode.dump);
        let program = match loader.parse_template(file.as_str(), &root_context) {
            Ok(res) => res,
            Err(err) => {
                loader.explain_error(&err)?;
                continue;
            }
        };

        for stmt in &program.code {
            println!("{stmt:?}");
        }

        if args.no_act {
            continue;
        }

        let sandbox = Sandbox::new(&["layers/empty".into()], "layers/tmp".into())?;
        let mut exec = Executor::new(sandbox);

        exec.run(&loader, &program)?;

        exec.finish()?;
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
