mod error;
mod executor;
mod loader;

pub use error::*;
pub use executor::*;
pub use loader::*;

use crate::dsl::Statement;

pub struct Program {
    pub code: Vec<Statement>,
}

impl Program {
    #[must_use]
    pub const fn new(code: Vec<Statement>) -> Self {
        Self { code }
    }
}

impl<'a> IntoIterator for &'a Program {
    type Item = &'a Statement;
    type IntoIter = <&'a Vec<Statement> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.code.iter()
    }
}

impl Program {
    pub fn iter(&self) -> std::slice::Iter<Statement> {
        self.code.iter()
    }
}
