use std::fs::File;
use std::io::{BufWriter, Write};
use std::process::Command;

use camino::Utf8PathBuf;
use indicatif::{ProgressBar, ProgressStyle};

use crate::dsl::{Instruction, Statement};
use crate::sandbox::Sandbox;
use crate::RaptorResult;

pub struct Executor {
    sandbox: Sandbox,
}

impl Executor {
    const BUFFER_SIZE: usize = 128 * 1024;

    const PROGRESS_STYLE: &str = "[{elapsed_precise}] {bar:40.cyan/blue} {bytes:>7}/{total_bytes:7} {binary_bytes_per_sec} {msg}";

    #[must_use]
    pub const fn new(sandbox: Sandbox) -> Self {
        Self { sandbox }
    }

    fn progress_bar(len: u64) -> ProgressBar {
        let style = ProgressStyle::with_template(Self::PROGRESS_STYLE)
            .unwrap()
            .progress_chars("#>-");

        indicatif::ProgressBar::new(len).with_style(style)
    }

    pub fn handle(&mut self, stmt: &Statement) -> RaptorResult<()> {
        match &stmt.inst {
            Instruction::From(inst) => {
                info!("{:?}", inst);
            }
            Instruction::Copy(inst) => {
                info!("{:?}", inst);
                let mut src = File::open(&inst.srcs[0])?;
                let fd = self.sandbox.create_file_handle(
                    &Utf8PathBuf::from(&inst.dest),
                    inst.chown.clone(),
                    inst.chmod,
                )?;

                let pb = Self::progress_bar(src.metadata()?.len());
                let mut dst = BufWriter::with_capacity(Self::BUFFER_SIZE, pb.wrap_write(fd));

                std::io::copy(&mut src, &mut dst)?;
            }
            Instruction::Render(inst) => {
                info!("{:?}", inst);
            }
            Instruction::Write(inst) => {
                info!("{:?}", inst);
                let mut fd = self.sandbox.create_file_handle(
                    &Utf8PathBuf::from(&inst.dest),
                    inst.chown.clone(),
                    inst.chmod,
                )?;
                fd.write_all(inst.body.as_bytes())?;
            }
            Instruction::Run(inst) => {
                debug!("{:?}", inst);
                self.sandbox.run(&inst.run)?;
            }

            Instruction::Invoke(inst) => {
                Command::new("echo").args(&inst.args).spawn()?.wait()?;
            }

            Instruction::Include(_) => unreachable!(),
        }

        Ok(())
    }

    pub fn finish(self) -> RaptorResult<()> {
        self.sandbox.close()
    }
}
