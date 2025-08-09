use std::fs::File;
use std::process::Command;

use camino::Utf8PathBuf;
use indicatif::{ProgressBar, ProgressStyle};
use minijinja::Value;

use crate::build::LayerInfo;
use crate::dsl::{Instruction, Program, ResolveArgs, Statement};
use crate::program::Loader;
use crate::sandbox::{Sandbox, SandboxExt};
use crate::util::io_fast_copy;
use crate::{template, RaptorResult};

pub struct Executor {
    sandbox: Sandbox,
    layer: LayerInfo,
}

impl Executor {
    const PROGRESS_STYLE: &str = "[{elapsed_precise}] {bar:40.cyan/blue} {bytes:>7}/{total_bytes:7} {binary_bytes_per_sec} {msg}";

    #[must_use]
    pub const fn new(sandbox: Sandbox, layer: LayerInfo) -> Self {
        Self { sandbox, layer }
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
            Instruction::From(_) => {}
            Instruction::Copy(inst) => {
                let srcname = stmt.origin.basedir()?.join(&inst.srcs[0]);
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
                let map = inst.args.clone().resolve_args(ctx)?;

                let srcname = stmt.origin.basedir()?.join(&inst.src);

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

            Instruction::Invoke(inst) => {
                Command::new(&inst.args[0])
                    .args(&inst.args[1..])
                    .env("RAPTOR_LAYER_NAME", self.layer.name())
                    .env("RAPTOR_LAYER_HASH", self.layer.hash())
                    .env("RAPTOR_LAYER_ID", self.layer.id())
                    .env("RAPTOR_BUILD_DIR", self.sandbox.get_root_dir())
                    .env("RAPTOR_TEMP_DIR", self.sandbox.get_temp_dir().unwrap())
                    .spawn()?
                    .wait()?;
            }

            Instruction::Env(inst) => {
                for env in &inst.env {
                    client.setenv(&env.key, &env.value)?;
                }
            }

            Instruction::Workdir(inst) => {
                client.chdir(&inst.dir)?;
            }

            Instruction::Include(_) => unreachable!(),
        }

        Ok(())
    }

    pub fn run(&mut self, loader: &Loader, program: &Program) -> RaptorResult<()> {
        program.traverse(&mut |stmt| {
            info!("{}", stmt.inst);
            if let Err(err) = self.handle(stmt, &program.ctx) {
                loader.explain_exec_error(stmt, &err)?;
                return Err(err);
            }
            Ok(())
        })
    }

    pub fn finish(mut self) -> RaptorResult<()> {
        self.sandbox.close()
    }
}
