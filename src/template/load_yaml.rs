use std::fs::File;

use minijinja::{Environment, Error, Value};

use crate::template::AdaptError;

#[allow(clippy::unnecessary_wraps)]
pub fn load_yaml(filename: &str) -> Result<Value, Error> {
    let file = File::open(filename).adapt_err("Failed to open file")?;
    let yml: serde_yml::Value = serde_yml::from_reader(&file).adapt_err("Failed to parse yaml")?;

    Ok(Value::from_serialize(yml))
}

pub fn add_functions(env: &mut Environment) {
    env.add_function("load_yaml", load_yaml);
}
