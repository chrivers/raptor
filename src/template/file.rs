use std::sync::Arc;

use camino::Utf8PathBuf;
use minijinja::value::{from_args, Object};
use minijinja::{Environment, Error, ErrorKind, State, Value};

#[derive(Debug)]
struct PathWrap(Utf8PathBuf);

impl Object for PathWrap {
    fn call_method(
        self: &Arc<Self>,
        _state: &State,
        method: &str,
        args: &[Value],
    ) -> Result<Value, Error> {
        () = from_args(args)?;
        match method {
            "exists" => Ok(Value::from(self.0.exists())),
            _ => Err(Error::from(ErrorKind::UnknownMethod)),
        }
    }
}

#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
pub fn path(name: &str) -> Result<Value, Error> {
    let obj = PathWrap(Utf8PathBuf::from(name));
    let value = Value::from_object(obj);

    Ok(value)
}

pub fn add_functions(env: &mut Environment) {
    env.add_function("path", path);
}
