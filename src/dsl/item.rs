use std::fmt::Debug;

use minijinja::Value;

use crate::dsl::{Instruction, Origin, Statement};
use crate::program::Program;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Item {
    Statement(Statement),
    Program(Program),
}

impl Item {
    #[must_use]
    pub const fn program(code: Vec<Self>, ctx: Value) -> Self {
        Self::Program(Program { code, ctx })
    }

    #[must_use]
    pub const fn statement(inst: Instruction, origin: Origin) -> Self {
        Self::Statement(Statement { inst, origin })
    }
}
