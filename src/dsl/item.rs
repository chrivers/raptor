use std::fmt::Debug;

use minijinja::Value;

use crate::dsl::{Instruction, Origin, Program, Statement};

#[derive(Clone, PartialEq, Eq)]
pub enum Item {
    Statement(Statement),
    Program(Program),
}

impl Item {
    #[must_use]
    pub fn program(code: impl IntoIterator<Item = Self>, ctx: Value) -> Self {
        Self::Program(Program {
            code: code.into_iter().collect(),
            ctx,
        })
    }

    #[must_use]
    pub const fn statement(inst: Instruction, origin: Origin) -> Self {
        Self::Statement(Statement { inst, origin })
    }
}

impl Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Program(prog) => prog.fmt(f),
            Self::Statement(stmt) => stmt.fmt(f),
        }
    }
}
