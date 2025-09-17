use std::borrow::Cow;
use std::sync::Arc;

use itertools::Itertools;
use minijinja::{Environment, Error, ErrorKind, Value, value::ValueKind};

const fn blacklisted(ch: char) -> bool {
    !matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '=' | '/' | ',' | '.' | '+')
}

fn escape(value: &str) -> String {
    if value.is_empty() {
        return String::from("\"\"");
    }

    if !value.contains(blacklisted) {
        return value.into();
    }

    let mut res = String::with_capacity(value.len() + 2);

    res.push('"');

    for c in value.chars() {
        match c {
            '\"' => res.push_str("\\\""),
            '\t' => res.push_str("\\t"),
            '\n' => res.push_str("\\n"),
            '\\' => res.push_str("\\\\"),
            c => res.push(c),
        }
    }

    res.push('"');

    res
}

fn error<T, D: Into<Cow<'static, str>>>(detail: D) -> Result<T, Error> {
    Err(Error::new(ErrorKind::BadEscape, detail))
}

#[allow(clippy::needless_pass_by_value)]
fn escape_sh(value: Value) -> Result<String, Error> {
    match value.kind() {
        ValueKind::Bool => {
            let b: bool = value.try_into()?;
            Ok(b.to_string())
        }
        ValueKind::Number => {
            let f: f64 = value.try_into()?;
            if f.is_nan() {
                error("Cannot escape floating point NaN")?;
            }
            if f.is_infinite() {
                error("Cannot escape floating point Infinity")?;
            }
            Ok(f.to_string())
        }
        ValueKind::String => {
            let val: Arc<str> = value.try_into()?;
            Ok(escape(&val))
        }
        ValueKind::Seq => {
            let val: Vec<_> = value.try_iter()?.map(escape_sh).try_collect()?;
            Ok(val.join(" "))
        }
        kind => error(format!("Cannot escape value: {value:?} of type {kind:?}"))?,
    }
}

pub fn add_filters(env: &mut Environment) {
    env.add_filter("sh", escape_sh);
    env.add_filter("escape_sh", escape_sh);
}
