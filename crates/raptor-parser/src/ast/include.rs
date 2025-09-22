use std::fmt::{self, Debug, Display};

use camino::Utf8Path;
use minijinja::Value;
use serde::Serialize;

use crate::ast::{Location, Origin};
use crate::print::Theme;
use crate::util::module_name::ModuleName;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Ident {
    pub name: String,
    pub origin: Origin,
}

impl Ident {
    #[must_use]
    pub const fn new(name: String, origin: Origin) -> Self {
        Self { name, origin }
    }
}

impl Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.name)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Lookup {
    pub expr: Box<Expression>,
    pub ident: Ident,
    pub origin: Origin,
}

impl Lookup {
    #[must_use]
    pub const fn new(expr: Box<Expression>, ident: Ident, origin: Origin) -> Self {
        Self {
            expr,
            ident,
            origin,
        }
    }

    pub fn make(expr: impl Into<Expression>, ident: impl Into<String>, origin: Origin) -> Self {
        Self {
            expr: Box::new(expr.into()),
            ident: Ident {
                name: ident.into(),
                origin: origin.clone(),
            },
            origin,
        }
    }
}

impl Display for Lookup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", &self.expr, self.ident)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Expression {
    Lookup(Lookup),
    Value(Value),
    Ident(Location<String>),
}

impl Expression {
    #[must_use]
    pub fn ident(name: &str, origin: Origin) -> Self {
        Self::Ident(Location::make(origin, name.to_string()))
    }
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

    pub fn lookup(
        name: impl AsRef<str>,
        expr: impl Into<Expression>,
        ident: impl Into<String>,
        origin: Origin,
    ) -> Self {
        Self {
            name: name.as_ref().to_string(),
            value: Expression::Lookup(Lookup::make(expr, ident, origin)),
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
            Self::Ident(i) => write!(f, "{}", i.inner),
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
