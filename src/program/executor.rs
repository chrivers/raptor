use std::fs::File;

use camino::Utf8PathBuf;
use indicatif::{ProgressBar, ProgressStyle};
use minijinja::Value;

use crate::dsl::Program;
use crate::program::{Loader, ResolveArgs};
use crate::sandbox::{Sandbox, SandboxExt};
use crate::util::io_fast_copy;
use crate::{RaptorResult, template};
use raptor_parser::ast::{Instruction, Statement};

pub struct Executor {
    sandbox: Sandbox,
}

impl Executor {
    const PROGRESS_STYLE: &str = "[{elapsed_precise}] {bar:40.cyan/blue} {bytes:>7}/{total_bytes:7} {binary_bytes_per_sec} {msg}";

    #[must_use]
    pub const fn new(sandbox: Sandbox) -> Self {
        Self { sandbox }
    }

    fn progress_bar(len: u64) -> ProgressBar {
        let style = ProgressStyle::with_template(Self::PROGRESS_STYLE)
            .unwrap()
            .progress_chars("#>-");

        ProgressBar::new(len).with_style(style)
    }

    fn handle(&mut self, stmt: &Statement, ctx: &Value) -> RaptorResult<()> {
        let client = self.sandbox.client();
        match &stmt.inst {
            // Code merging and mount instruction have nothing to execute
            Instruction::From(_)
            | Instruction::Include(_)
            | Instruction::Mount(_)
            | Instruction::Entrypoint(_)
            | Instruction::Cmd(_) => {}

            Instruction::Copy(inst) => {
                let srcname = stmt.origin.path_for(&inst.srcs[0])?;
                let src = File::open(&srcname)?;
                let fd = client.create_file(
                    &Utf8PathBuf::from(&inst.dest),
                    inst.chown.clone(),
                    inst.chmod,
                )?;

                let pb = Self::progress_bar(src.metadata()?.len());
                let dst = pb.wrap_write(fd);
                io_fast_copy(src, dst)?;
            }

            Instruction::Render(inst) => {
                let map = ctx.resolve_args(&inst.args)?;

                let srcname = stmt.origin.path_for(&inst.src)?;

                let source = template::make_environment()?
                    .get_template(srcname.as_str())
                    .and_then(|tmpl| tmpl.render(Value::from(map)))
                    .map(|src| src + "\n")?;

                client.write_file(
                    &inst.dest,
                    inst.chown.clone(),
                    inst.chmod,
                    source.as_bytes(),
                )?;
            }

            Instruction::Write(inst) => {
                client.write_file(
                    &inst.dest,
                    inst.chown.clone(),
                    inst.chmod,
                    inst.body.as_bytes(),
                )?;
            }

            Instruction::Mkdir(inst) => {
                client.mkdir(&inst.dest, inst.chown.clone(), inst.chmod, inst.parents)?;
            }

            Instruction::Run(inst) => {
                client.run(&inst.run)?;
            }

            Instruction::Env(inst) => {
                for env in &inst.env {
                    client.setenv(&env.key, &env.value)?;
                }
            }

            Instruction::Workdir(inst) => {
                client.chdir(inst.dir.as_str())?;
            }
        }

        Ok(())
    }

    pub fn run(&mut self, loader: &Loader, program: &Program) -> RaptorResult<()> {
        program.traverse(&mut |stmt| {
            info!("{}", stmt.inst);
            self.handle(stmt, &program.ctx).or_else(|err| {
                loader.explain_exec_error(stmt, &err, &[])?;
                Err(err)
            })
        })
    }

    pub fn finish(mut self) -> RaptorResult<()> {
        self.sandbox.close()
    }
}
