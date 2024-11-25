use std::fmt::{self, Display};

use colored::Colorize;
use minijinja::Value;

use crate::dsl::{Item, Origin};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub code: Vec<Item>,
    pub ctx: Value,
    pub origin: Origin,
}

impl Program {
    #[must_use]
    pub const fn new(code: Vec<Item>, ctx: Value, origin: Origin) -> Self {
        Self { code, ctx, origin }
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn dump(f: &mut fmt::Formatter<'_>, program: &Program, level: usize) -> fmt::Result {
            let indent = " ".repeat(level * 4);
            writeln!(f, "{indent}{}{}", "# file ".dimmed(), program.origin.path)?;
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
