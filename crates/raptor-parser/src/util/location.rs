use std::fmt::Display;

use crate::ast::Origin;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Location<T> {
    pub origin: Origin,
    pub inner: T,
}

impl<T> Location<T> {
    pub fn origin(&self) -> Origin {
        self.origin.clone()
    }

    pub const fn make(origin: Origin, inner: T) -> Self {
        Self { origin, inner }
    }
}

impl<T: Display> Display for Location<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}
