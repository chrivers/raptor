use std::fmt::{self, Display};

use colored::Colorize;
use minijinja::Value;

use crate::dsl::Item;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub code: Vec<Item>,
    pub ctx: Value,
}

impl Program {
    #[must_use]
    pub const fn new(code: Vec<Item>, ctx: Value) -> Self {
        Self { code, ctx }
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
        for item in &self.code {
            match item {
                Item::Statement(stmt) => {
                    writeln!(f, "{}", stmt.inst)?;
                }
                Item::Program(prog) => {
                    writeln!(f, "{}", "# include".dimmed())?;
                    write!(f, "{prog}")?;
                }
            }
        }
        Ok(())
    }
}

impl Program {
    pub fn iter(&self) -> std::slice::Iter<Item> {
        self.code.iter()
    }
}
