use std::fs::File;

use camino::Utf8Path;
use minijinja::{Environment, Error, State, Value};

use crate::template::AdaptError;

#[allow(clippy::unnecessary_wraps)]
pub fn load_yaml(state: &State, filename: &str) -> Result<Value, Error> {
    let path = Utf8Path::new(state.name()).parent().unwrap();
    let file = File::open(path.join(filename)).adapt_err("Failed to open file")?;
    let yml: serde_yml::Value = serde_yml::from_reader(&file).adapt_err("Failed to parse yaml")?;

    Ok(Value::from_serialize(yml))
}

pub fn add_functions(env: &mut Environment) {
    env.add_function("load_yaml", load_yaml);
}
