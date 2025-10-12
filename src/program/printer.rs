use crate::RaptorResult;
use crate::dsl::Program;

#[derive(Default)]
pub struct PrintExecutor {}

impl PrintExecutor {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    pub fn run(&self, program: &Program) -> RaptorResult<()> {
        program.traverse(&mut |stmt| {
            info!("{}", stmt.inst);
            Ok(())
        })
    }
}
