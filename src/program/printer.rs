use crate::dsl::Program;
use crate::program::Loader;
use crate::RaptorResult;

#[derive(Default)]
pub struct PrintExecutor {}

impl PrintExecutor {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    pub fn run(&mut self, _loader: &Loader, program: &Program) -> RaptorResult<()> {
        program.traverse(&mut |stmt| {
            info!("{}", stmt.inst);
        });
        Ok(())
    }
}
