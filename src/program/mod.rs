mod error;
mod executor;
mod loader;

pub use error::*;
pub use executor::*;
pub use loader::*;

use crate::dsl::Statement;

pub struct Program(pub Vec<Statement>);

impl<'a> IntoIterator for &'a Program {
    type Item = &'a Statement;
    type IntoIter = <&'a Vec<Statement> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Program {
    pub fn iter(&self) -> std::slice::Iter<Statement> {
        self.0.iter()
    }
}
