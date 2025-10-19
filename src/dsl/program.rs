use std::fmt::{self, Display};

use camino::Utf8PathBuf;
use colored::Colorize;
use minijinja::Value;

use raptor_parser::ast::{
    FromSource, InstCmd, InstEntrypoint, InstMount, Instruction, Origin, Statement,
};

use crate::RaptorResult;
use crate::dsl::Item;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Program {
    pub code: Vec<Item>,
    pub ctx: Value,
    pub path: Utf8PathBuf,
}

impl Program {
    #[must_use]
    pub const fn new(code: Vec<Item>, ctx: Value, path: Utf8PathBuf) -> Self {
        Self { code, ctx, path }
    }

    pub fn traverse(
        &self,
        visitor: &mut impl FnMut(&Statement) -> RaptorResult<()>,
    ) -> RaptorResult<()> {
        for stmt in &self.code {
            match stmt {
                Item::Statement(stmt) => visitor(stmt)?,
                Item::Program(prog) => {
                    prog.traverse(visitor)?;
                }
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn from(&self) -> Option<(&FromSource, &Origin)> {
        for item in &self.code {
            if let Item::Statement(Statement {
                inst: Instruction::From(inst),
                origin,
            }) = item
            {
                return Some((&inst.from, origin));
            }
        }

        None
    }

    #[must_use]
    pub fn cmd(&self) -> Option<&InstCmd> {
        for item in &self.code {
            if let Item::Statement(Statement {
                inst: Instruction::Cmd(inst),
                ..
            }) = item
            {
                return Some(inst);
            }
        }

        None
    }

    #[must_use]
    pub fn entrypoint(&self) -> Option<&InstEntrypoint> {
        for item in &self.code {
            if let Item::Statement(Statement {
                inst: Instruction::Entrypoint(inst),
                ..
            }) = item
            {
                return Some(inst);
            }
        }

        None
    }

    #[must_use]
    pub fn mounts(&self) -> Vec<&InstMount> {
        let mut mounts = vec![];

        for item in &self.code {
            if let Item::Statement(Statement {
                inst: Instruction::Mount(inst),
                ..
            }) = item
            {
                mounts.push(inst);
            }
        }

        mounts
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn dump(f: &mut fmt::Formatter, program: &Program, level: usize) -> fmt::Result {
            let indent = if f.alternate() {
                &" ".repeat(level * 4)
            } else {
                ""
            };
            writeln!(f, "{indent}{}{}", "# file ".dimmed(), program.path)?;
            for item in &program.code {
                match item {
                    Item::Statement(stmt) => {
                        writeln!(f, "{indent}{}", stmt.inst)?;
                    }
                    Item::Program(prog) => {
                        dump(f, prog, level + 1)?;
                    }
                }
            }
            Ok(())
        }

        dump(f, self, 0)
    }
}
