use std::fmt::{Debug, Display};

use minijinja::Value;

#[derive(Debug)]
pub struct Lookup {
    pub path: Vec<String>,
}

impl Lookup {
    #[must_use]
    pub const fn new(path: Vec<String>) -> Self {
        Self { path }
    }
}

impl Display for Lookup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.path.join("."))
    }
}

#[derive(Debug)]
pub enum IncludeArgValue {
    Lookup(Lookup),
    Value(Value),
}

#[derive(Debug)]
pub struct IncludeArg {
    pub name: String,
    pub value: IncludeArgValue,
}

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
