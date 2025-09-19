use std::collections::HashMap;

use minijinja::Value;

use raptor_parser::ast::{IncludeArg, IncludeArgValue};
use raptor_parser::value::Value as RaptorValue;

use crate::{RaptorError, RaptorResult};

pub trait ResolveArg {
    fn resolve(&self, arg: IncludeArgValue) -> RaptorResult<Value>;
}

pub trait ResolveArgs {
    fn resolve_args<'a>(&'a self, args: &'a [IncludeArg]) -> RaptorResult<HashMap<&'a str, Value>>;
}

impl ResolveArg for Value {
    fn resolve(&self, arg: IncludeArgValue) -> RaptorResult<Value> {
        match arg {
            IncludeArgValue::Lookup(lookup) => {
                let mut val = self.get_attr(&lookup.path.parts()[0])?;
                if val.is_undefined() {
                    return Err(RaptorError::UndefinedVarError(
                        lookup.path.parts()[0].to_string(),
                        lookup.origin.clone(),
                    ));
                }

                for name in &lookup.path.parts()[1..] {
                    val = val.get_attr(name)?;
                    if val.is_undefined() {
                        return Err(RaptorError::UndefinedVarError(
                            name.into(),
                            lookup.origin.clone(),
                        ));
                    }
                }

                Ok(val)
            }

            IncludeArgValue::Value(val) => match val {
                RaptorValue::Bool(v) => Ok(v.into()),
                RaptorValue::Number(v) => Ok(v.into()),
                RaptorValue::String(v) => Ok(v.into()),
            },
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
