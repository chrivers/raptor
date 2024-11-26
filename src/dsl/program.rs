use std::fmt::{self, Display};

use camino::Utf8PathBuf;
use colored::Colorize;
use minijinja::Value;

use crate::dsl::{Item, Statement};

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

    pub fn traverse(&self, visitor: &mut impl FnMut(&Statement)) {
        for stmt in &self.code {
            match stmt {
                Item::Statement(stmt) => visitor(stmt),
                Item::Program(prog) => {
                    prog.traverse(visitor);
                }
            }
        }
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
            let indent = " ".repeat(level * 4);
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
