use std::collections::HashMap;

use minijinja::Value;

use raptor_parser::ast::{Expression, IncludeArg};

use crate::{RaptorError, RaptorResult};

pub trait ResolveArg {
    fn resolve(&self, arg: Expression) -> RaptorResult<Value>;
}

pub trait ResolveArgs {
    fn resolve_args<'a>(&'a self, args: &'a [IncludeArg]) -> RaptorResult<HashMap<&'a str, Value>>;
}

impl ResolveArg for Value {
    fn resolve(&self, arg: Expression) -> RaptorResult<Value> {
        match arg {
            Expression::Ident(ident) => {
                let val = self.get_attr(&ident.inner)?;

                if val.is_undefined() {
                    let origin = ident.origin();
                    return Err(RaptorError::UndefinedVarError(ident.inner, origin));
                }

                Ok(val)
            }
            Expression::Lookup(lookup) => {
                let expr = self.resolve(*lookup.expr)?;

                let val = expr.get_attr(&lookup.ident.name)?;

                if val.is_undefined() {
                    return Err(RaptorError::UndefinedVarError(
                        lookup.ident.name,
                        lookup.origin.clone(),
                    ));
                }

                Ok(val)
            }

            Expression::Value(val) => Ok(val),
        }
    }
}

impl ResolveArgs for Value {
    fn resolve_args<'a>(&'a self, args: &'a [IncludeArg]) -> RaptorResult<HashMap<&'a str, Value>> {
        args.iter()
            .map(|IncludeArg { name, value }| Ok((name.as_str(), self.resolve(value.clone())?)))
            .collect()
    }
}
