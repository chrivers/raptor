use std::collections::HashMap;
use std::fmt::{self, Debug, Display};

use minijinja::Value;
use serde::Serialize;

use crate::dsl::Origin;
use crate::print::Theme;
use crate::util::module_name::ModuleName;
use crate::{RaptorError, RaptorResult};

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
pub enum IncludeArgValue {
    Lookup(Lookup),
    Value(Value),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
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
                path: ModuleName::new(path.iter().map(ToString::to_string).collect()),
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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InstInclude {
    pub src: String,
    pub args: Vec<IncludeArg>,
}

impl IncludeArgValue {
    pub fn resolve(self, ctx: &Value) -> RaptorResult<Value> {
        match self {
            Self::Lookup(lookup) => {
                let mut val = ctx.get_attr(&lookup.path.parts()[0])?;
                if val.is_undefined() {
                    return Err(RaptorError::UndefinedVarError(
                        lookup.path.parts()[0].to_string(),
                        lookup.origin,
                    ));
                }
                for name in &lookup.path.parts()[1..] {
                    val = val.get_attr(name)?;
                    if val.is_undefined() {
                        return Err(RaptorError::UndefinedVarError(name.into(), lookup.origin));
                    }
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}={}", self.name, self.value)?;
        Ok(())
    }
}

impl Display for IncludeArgValue {
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
        f.src(&self.src)?;
        for arg in &self.args {
            f.include_arg(arg)?;
        }
        Ok(())
    }
}
