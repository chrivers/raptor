use std::sync::Arc;

use camino::Utf8PathBuf;
use minijinja::value::{Object, from_args};
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
            "is_dir" => Ok(Value::from(self.0.is_dir())),
            "is_file" => Ok(Value::from(self.0.is_file())),
            "is_symlink" => Ok(Value::from(self.0.is_symlink())),
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
