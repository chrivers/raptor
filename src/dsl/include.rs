use std::fmt::{Debug, Display};

use minijinja::Value;

use crate::dsl::Origin;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lookup {
    pub path: Vec<String>,
    pub origin: Origin,
}

impl Lookup {
    #[must_use]
    pub const fn new(path: Vec<String>, origin: Origin) -> Self {
        Self { path, origin }
    }
}

impl Display for Lookup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.path.join("."))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IncludeArgValue {
    Lookup(Lookup),
    Value(Value),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncludeArg {
    pub name: String,
    pub value: IncludeArgValue,
}

#[derive(Clone, PartialEq, Eq)]
pub struct InstInclude {
    pub src: String,
    pub args: Vec<IncludeArg>,
}

impl Debug for InstInclude {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "INCLUDE {}", self.src)?;
        for arg in &self.args {
            write!(f, " {arg:?}")?;
        }
        Ok(())
    }
}
