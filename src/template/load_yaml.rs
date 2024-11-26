use std::fs::File;

use minijinja::{Environment, Error, ErrorKind, Value};

#[allow(clippy::unnecessary_wraps)]
pub fn load_yaml(filename: &str) -> Result<Value, Error> {
    let file = File::open(filename).map_err(|err| {
        Error::new(ErrorKind::InvalidOperation, "Failed to open file").with_source(err)
    })?;
    let yml: serde_yml::Value = serde_yml::from_reader(&file).map_err(|err| {
        Error::new(ErrorKind::InvalidOperation, "Failed to parse yaml").with_source(err)
    })?;
    Ok(Value::from_serialize(yml))
}

pub fn add_functions(env: &mut Environment) {
    env.add_function("load_yaml", load_yaml);
}
