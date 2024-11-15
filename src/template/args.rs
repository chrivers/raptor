use std::sync::{Arc, Mutex};

use clap::{ArgMatches, Command};
use minijinja::value::{from_args, Enumerator, Kwargs, Object, ObjectRepr};
use minijinja::{Environment, Error, State, Value};

use serde::de::Error as _;

use crate::util::kwargs::KwargsExt;

#[derive(Debug)]
struct Arg {
    name: String,
    required: bool,
    help: Option<String>,
    default: Option<Value>,
}

#[derive(Debug)]
struct Args(Mutex<Vec<Arg>>);

#[derive(Debug)]
struct MatchWrapper(ArgMatches);

impl Object for MatchWrapper {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Map
    }

    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        self.0
            .get_one(&key.to_string())
            .map(|val: &String| Value::from_serialize(val))
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Values(
            self.0
                .ids()
                .map(|id| Value::from(id.as_str()))
                .collect::<Vec<Value>>(),
        )
    }

    fn enumerator_len(self: &Arc<Self>) -> Option<usize> {
        None
    }
}

impl Object for Args {
    fn call_method(
        self: &Arc<Self>,
        _state: &State,
        method: &str,
        args: &[Value],
    ) -> Result<Value, Error> {
        let (args, kwargs) = from_args::<(&[Value], Kwargs)>(args)?;
        match method {
            "param" => {
                let required = kwargs.get_or_default("required", false)?;
                let help: Option<String> = kwargs.get_option("help")?;
                let default: Option<Value> = kwargs.get_option("default")?;

                let (name,): (String,) = from_args(args)?;

                kwargs.assert_all_used()?;

                self.0.lock().unwrap().push(Arg {
                    name,
                    required,
                    help,
                    default,
                });

                Ok(Value::UNDEFINED)
            }

            "parse" => {
                let mut cmd = Command::new("foo")
                    .no_binary_name(true)
                    .disable_help_flag(true)
                    .disable_colored_help(true);

                let lock = self.0.lock().unwrap();
                for arg in lock.iter() {
                    let mut a = clap::Arg::new(&arg.name).required(arg.required);
                    if let Some(h) = &arg.help {
                        a = a.help(h);
                    }
                    if let Some(default) = &arg.default {
                        a = a.default_value(default.to_string());
                    }
                    cmd = cmd.arg(a);
                }

                let (value,): (Value,) = from_args(args)?;

                let res = cmd
                    .try_get_matches_from(value.try_iter()?.map(|x| x.to_string()))
                    .map_err(|e| Error::custom(format!("\n{e}")))?;

                drop(lock);

                Ok(Value::from_object(MatchWrapper(res)))
            }
            _ => todo!(),
        }
    }
}

#[allow(clippy::unnecessary_wraps)]
pub fn args() -> Result<Value, Error> {
    Ok(Value::from_object(Args(Mutex::new(vec![]))))
}

pub fn add_functions(env: &mut Environment) {
    env.add_function("Args", args);
}
