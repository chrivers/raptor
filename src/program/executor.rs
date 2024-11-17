use std::process::Command;

use crate::{
    dsl::{Instruction, Statement},
    sandbox::Sandbox,
    RaptorResult,
};

pub struct Executor {
    sandbox: Sandbox,
}

impl Executor {
    #[must_use]
    pub const fn new(sandbox: Sandbox) -> Self {
        Self { sandbox }
    }

    pub fn handle(&mut self, stmt: &Statement) -> RaptorResult<()> {
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
                /* self.sandbox.run(&inst.run)?; */
            }

            Instruction::Invoke(inst) => {
                Command::new("echo").args(&inst.args).spawn()?.wait()?;
            }

            Instruction::Include(_) => unreachable!(),
        }

        Ok(())
    }

    pub fn finish(mut self) -> RaptorResult<()> {
        self.sandbox.close()
    }
}
