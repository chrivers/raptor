use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use camino::Utf8Path;
use minijinja::Value;

use crate::dsl::Program;
use raptor_parser::ast::{Instruction, Origin, Statement};

#[derive(Clone, PartialEq, Eq)]
pub enum Item {
    Statement(Statement),
    Program(Arc<Program>),
}

impl Item {
    #[must_use]
    pub fn program(
        code: impl IntoIterator<Item = Self>,
        ctx: Value,
        path: impl AsRef<Utf8Path>,
    ) -> Self {
        Self::Program(Arc::new(Program {
            code: code.into_iter().collect(),
            ctx,
            path: path.as_ref().into(),
        }))
    }

    #[must_use]
    pub const fn statement(inst: Instruction, origin: Origin) -> Self {
        Self::Statement(Statement { inst, origin })
    }
}

impl Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Program(prog) => prog.fmt(f),
            Self::Statement(stmt) => stmt.fmt(f),
        }
    }
}

impl Hash for Item {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Statement(statement) => statement.inst.hash(state),
            Self::Program(program) => {
                program.code.hash(state);
                program.ctx.hash(state);
            }
        }
    }
}
