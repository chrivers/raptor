use std::collections::HashMap;
use std::process::Command;

use annotate_snippets::{Level, Renderer, Snippet};
use camino::Utf8PathBuf;
use clap::Parser as _;
use log::{debug, error, info};
use minijinja::{context, Environment, Value};

use raptor::dsl::{IncludeArgValue, Instruction, Origin, Statement};
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

fn show_error_context(
    name: &str,
    origin: &str,
    description: &str,
    err_line: usize,
) -> RaptorResult<()> {
    const CONTEXT_LINES: usize = 3;

    let raw = std::fs::read_to_string(name)?;
    let lines = raw.lines().collect::<Vec<&str>>();
    let line = err_line - 1;
    let line1 = line.saturating_sub(CONTEXT_LINES);
    let line2 = (line + 1 + CONTEXT_LINES).min(lines.len());
    let snippet = &lines[line1..line2].join("\n");
    let span1 = lines[line1..line]
        .iter()
        .map(|q| q.as_bytes().len())
        .sum::<usize>()
        + (line - line1);
    let span2 = span1 + lines[line].as_bytes().len();

    let message = Level::Error
        .title("Error while parsing raptor file")
        .snippet(
            Snippet::source(snippet)
                .line_start(line1 + 1)
                .origin(origin)
                .fold(false)
                .annotation(Level::Error.span(span1..span2).label(description)),
        );

    let renderer = Renderer::styled();
    anstream::println!("{}", renderer.render(message));

    Ok(())
}

struct Engine<'source> {
    env: Environment<'source>,
    sandbox: Sandbox,
    files: Vec<Origin>,
}

impl<'source> Engine<'source> {
    pub fn new(env: Environment<'source>, sandbox: Sandbox) -> Self {
        Self {
            env,
            files: vec![],
            sandbox,
        }
    }

    fn handle(&mut self, stmt: &Statement, rctx: &Value) -> RaptorResult<()> {
        match &stmt.inst {
            Instruction::From(inst) => {
                info!("{:?}", inst);
            }
            Instruction::Copy(inst) => {
                info!("{:?}", inst);
            }
            Instruction::Render(inst) => {
                info!("{:?}", inst);
            }
            Instruction::Write(inst) => {
                info!("{:?}", inst);
            }
            Instruction::Run(inst) => {
                debug!("{:?}", inst);
                self.sandbox.run(&inst.run)?;
            }

            Instruction::Include(inst) => {
                let mut map = HashMap::new();
                for arg in &inst.args {
                    match &arg.value {
                        IncludeArgValue::Lookup(lookup) => {
                            let name = &lookup.path[0];
                            let val = rctx.get_attr(name)?;
                            /* let val = rctx.get_value(&Value::from(name)).ok_or( */
                            /*     RaptorError::InstNameNotFound(Box::new(inst.clone()), name.clone()), */
                            /* )?; */
                            map.insert(arg.name.clone(), val.clone());
                        }
                        IncludeArgValue::Value(val) => {
                            map.insert(arg.name.clone(), Value::from_serialize(val));
                        }
                    }
                }
                let ctx = Value::from(map);
                self.files.push(stmt.origin.clone());
                self.execute_template(&inst.src, &ctx)?;
                self.files.pop();
            }

            Instruction::Invoke(inst) => {
                Command::new("echo").args(&inst.args).spawn()?.wait()?;
            }
        }

        Ok(())
    }

    fn parse_template(&mut self, name: impl AsRef<str>, ctx: &Value) -> RaptorResult<String> {
        match self
            .env
            .get_template(name.as_ref())
            .and_then(|tmpl| tmpl.render(ctx))
        {
            Ok(res) => Ok(res),
            Err(err) => {
                if let Some(err_line) = err.line() {
                    let description = err.to_string();
                    show_error_context(name.as_ref(), err.name().unwrap(), &description, err_line)?;
                } else {
                    error!("{err}");
                    let mut err2 = &err as &dyn std::error::Error;
                    while let Some(next_err) = err2.source() {
                        eprintln!();
                        eprintln!("caused by: {:#}", next_err);
                        err2 = next_err;
                    }
                }

                Err(err)?
            }
        }
    }

    fn execute_template(&mut self, path: &str, ctx: &Value) -> RaptorResult<()> {
        let res = match self.parse_template(path, ctx) {
            Ok(res) => res,
            Err(err) => {
                info!("foo: {path} {:?}", &self.files);
                show_error_context(path, "", &err.to_string(), 0)?;
                Err(err)?
            }
        };

        let source = res + "\n";
        let ast = raptor::parser::ast::parse(path, &source)?;

        for inst in ast {
            self.handle(&inst, ctx)?;
        }

        Ok(())
    }

    pub fn finish(mut self) -> RaptorResult<()> {
        self.sandbox.close()
    }
}

fn raptor() -> RaptorResult<()> {
    let args = Cli::parse();

    let root_context = context!();

    for file in args.input {
        let spawn = Sandbox::new(&["layers/build", "layers/adjust"])?;
        let mut eng = Engine::new(template::make_environment()?, spawn);
        let source = eng.parse_template(&file, &root_context)?;

        if args.mode.dump {
            println!("{source}");
        }

        if args.no_act {
            continue;
        }

        let source = source + "\n";
        let ast = raptor::parser::ast::parse(file.as_str(), &source)?;

        for inst in ast {
            eng.handle(&inst, &root_context)?;
        }
        eng.finish()?;
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
