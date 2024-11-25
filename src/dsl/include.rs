use std::collections::HashMap;
use std::fmt::{self, Debug, Display};

use minijinja::Value;
use serde::Serialize;

use crate::dsl::Origin;
use crate::print::Theme;
use crate::{RaptorError, RaptorResult};

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl IncludeArg {
    pub fn make(name: impl AsRef<str>, value: IncludeArgValue) -> Self {
        Self {
            name: name.as_ref().to_string(),
            value,
        }
    }

    pub fn lookup(name: impl AsRef<str>, path: &[impl ToString], origin: Origin) -> Self {
        Self {
            name: name.as_ref().to_string(),
            value: IncludeArgValue::Lookup(Lookup {
                path: path.iter().map(ToString::to_string).collect(),
                origin,
            }),
        }
    }

    pub fn value(name: impl AsRef<str>, value: impl Serialize) -> Self {
        Self {
            name: name.as_ref().to_string(),
            value: IncludeArgValue::Value(Value::from_serialize(value)),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct InstInclude {
    pub src: String,
    pub args: Vec<IncludeArg>,
}

impl IncludeArgValue {
    pub fn resolve(self, ctx: &Value) -> RaptorResult<Value> {
        match self {
            Self::Lookup(lookup) => {
                let name = &lookup.path[0];
                let val = ctx.get_attr(name)?;
                if val.is_undefined() {
                    return Err(RaptorError::UndefinedVarError(name.into(), lookup.origin));
                }
                Ok(val)
            }
            Self::Value(val) => Ok(val),
        }
    }
}

pub trait ResolveArgs {
    fn resolve_args(self, ctx: &Value) -> RaptorResult<HashMap<String, Value>>;
}

impl ResolveArgs for Vec<IncludeArg> {
    fn resolve_args(self, ctx: &Value) -> RaptorResult<HashMap<String, Value>> {
        self.into_iter()
            .map(|IncludeArg { name, value }| Ok((name, value.resolve(ctx)?)))
            .collect()
    }
}

impl Display for IncludeArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.name, self.value)?;
        Ok(())
    }
}

impl Display for IncludeArgValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lookup(l) => write!(f, "{l}"),
            Self::Value(v) => write!(f, "{v:?}"),
        }
    }
}

impl Display for InstInclude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.keyword("INCLUDE")?;
        for arg in &self.args {
            f.include_arg(arg)?;
        }
        Ok(())
    }
}

impl Debug for InstInclude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "INCLUDE {}", self.src)?;
        for arg in &self.args {
            write!(f, " {arg:?}")?;
        }
        Ok(())
    }
}
