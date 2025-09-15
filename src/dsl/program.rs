use std::fmt::{self, Display};

use camino::{Utf8Path, Utf8PathBuf};
use colored::Colorize;
use minijinja::Value;

use crate::dsl::{FromSource, InstMount, Instruction, Item, Statement};
use crate::util::SafeParent;
use crate::RaptorResult;

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
    pub fn from(&self) -> Option<&FromSource> {
        for item in &self.code {
            if let Item::Statement(Statement {
                inst: Instruction::From(inst),
                ..
            }) = item
            {
                return Some(&inst.from);
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

    pub fn path_for(&self, path: impl AsRef<Utf8Path>) -> RaptorResult<Utf8PathBuf> {
        Ok(self.path.try_parent()?.join(path.as_ref()))
    }
}

impl IntoIterator for Program {
    type Item = Item;
    type IntoIter = <Vec<Item> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.code.into_iter()
    }
}

impl<'a> IntoIterator for &'a Program {
    type Item = &'a Item;
    type IntoIter = <&'a Vec<Item> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.code.iter()
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

impl Program {
    pub fn iter(&self) -> std::slice::Iter<Item> {
        self.code.iter()
    }
}
