use std::fmt::{self, Debug, Display};

use camino::Utf8Path;
use minijinja::Value;
use serde::Serialize;

use crate::ast::Origin;
use crate::print::Theme;
use crate::util::module_name::ModuleName;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Lookup {
    pub path: ModuleName,
    pub origin: Origin,
}

impl Lookup {
    #[must_use]
    pub const fn new(path: ModuleName, origin: Origin) -> Self {
        Self { path, origin }
    }
}

impl Display for Lookup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.path)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Expression {
    Lookup(Lookup),
    Value(Value),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct IncludeArg {
    pub name: String,
    pub value: Expression,
}

impl IncludeArg {
    pub fn make(name: impl AsRef<str>, value: Expression) -> Self {
        Self {
            name: name.as_ref().to_string(),
            value,
        }
    }

    pub fn lookup(name: impl AsRef<str>, path: &[impl ToString], origin: Origin) -> Self {
        Self {
            name: name.as_ref().to_string(),
            value: Expression::Lookup(Lookup {
                path: ModuleName::new(path.iter().map(ToString::to_string).collect()),
                origin,
            }),
        }
    }

    pub fn value(name: impl AsRef<str>, value: impl Serialize) -> Self {
        Self {
            name: name.as_ref().to_string(),
            value: Expression::Value(Value::from_serialize(value)),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InstInclude {
    pub src: ModuleName,
    pub args: Vec<IncludeArg>,
}

impl Display for IncludeArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}={}", self.name, self.value)?;
        Ok(())
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Lookup(l) => write!(f, "{l}"),
            Self::Value(v) => write!(f, "{v:?}"),
        }
    }
}

impl Display for InstInclude {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.keyword("INCLUDE")?;
        f.src(Utf8Path::new(&self.src.to_string()))?;
        for arg in &self.args {
            f.include_arg(arg)?;
        }
        Ok(())
    }
}
