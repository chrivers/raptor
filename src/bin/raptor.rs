use camino::Utf8PathBuf;
use clap::Parser as _;
use log::error;

use minijinja::{context, ErrorKind};
use raptor::dsl::Origin;
use raptor::program::{show_error_context, show_jinja_error_context, Executor, Loader};
use raptor::sandbox::Sandbox;
use raptor::{template, RaptorError, RaptorResult};

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

fn show_include_stack(origins: &[Origin]) -> RaptorResult<()> {
    for org in origins {
        show_error_context(
            &org.path,
            "Error while evaluating INCLUDE",
            "(included here)",
            org.span.clone(),
        )?;
    }

    Ok(())
}

fn raptor() -> RaptorResult<()> {
    let args = Cli::parse();

    let root_context = context!();

    for file in args.input {
        let mut loader = Loader::new(template::make_environment()?, args.mode.dump);
        let statements = match loader.parse_template(file.as_str(), &root_context) {
            Ok(res) => res,
            Err(RaptorError::ScriptError(desc, origin)) => {
                show_include_stack(loader.origins())?;
                show_error_context(&origin.path, "Script Error", &desc, origin.span.clone())?;
                continue;
            }
            Err(RaptorError::MinijinjaError(err)) => {
                match err.kind() {
                    ErrorKind::BadInclude => {
                        let mut origins = loader.origins().to_vec();
                        if let Some(last) = origins.pop() {
                            show_include_stack(&origins)?;

                            show_error_context(
                                &last.path,
                                "Error while evaluating INCLUDE",
                                err.detail().unwrap_or("error"),
                                err.range().unwrap_or(last.span.clone()),
                            )?;
                        } else {
                            error!("Cannot provide error context: {err}");
                        }
                    }
                    _ => {
                        show_include_stack(loader.origins())?;
                        show_jinja_error_context(&err)?;
                    }
                }
                continue;
            }
            Err(RaptorError::PestError(err)) => {
                error!("Failed to parse file: {err}");
                continue;
            }
            Err(err) => panic!("{err}"),
        };

        for stmt in &statements {
            println!("{:?} {:?}", stmt.origin, stmt.inst);
        }

        if args.no_act {
            continue;
        }

        let sandbox = Sandbox::new(&["layers/build", "layers/adjust"])?;
        let mut exec = Executor::new(sandbox);

        for stmt in &statements {
            exec.handle(stmt)?;
        }

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
