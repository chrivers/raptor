mod args;
mod escape;
mod file;
mod header;
mod load_yaml;
mod log;

use std::borrow::Cow;

use minijinja::syntax::SyntaxConfig;
use minijinja::{Environment, Error, ErrorKind, UndefinedBehavior};

use crate::RaptorResult;

trait AdaptError<T> {
    fn adapt_err(self, msg: impl Into<Cow<'static, str>>) -> Result<T, Error>;
}

impl<T, E> AdaptError<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn adapt_err(self, msg: impl Into<Cow<'static, str>>) -> Result<T, Error> {
        self.map_err(|err| Error::new(ErrorKind::InvalidOperation, msg).with_source(err))
    }
}

pub fn make_environment<'a>() -> RaptorResult<Environment<'a>> {
    let mut env = Environment::new();
    env.set_debug(true);
    env.set_keep_trailing_newline(true);
    env.set_undefined_behavior(UndefinedBehavior::Strict);

    env.set_loader(|name| {
        Ok(Some(
            std::fs::read_to_string(name)
                .map(|mut data| {
                    // file must end in newline, to avoid minijinja
                    // glueing the included file together incorrectly
                    if !data.ends_with('\n') {
                        data.push('\n');
                    }
                    data
                })
                .map_err(|e| {
                    Error::new(
                        ErrorKind::BadInclude,
                        format!("Could not open [{name}]: {e}"),
                    )
                })?,
        ))
    });

    env.set_path_join_callback(|name, parent| {
        let mut rv = parent.split('/').collect::<Vec<_>>();
        rv.pop();
        name.split('/').for_each(|segment| match segment {
            "." => {}
            ".." => drop(rv.pop()),
            _ => rv.push(segment),
        });
        rv.join("/").into()
    });

    env.set_syntax(
        SyntaxConfig::builder()
            .line_statement_prefix("$ ")
            .line_comment_prefix("#")
            .build()?,
    );

    log::add_functions(&mut env);
    file::add_functions(&mut env);
    args::add_functions(&mut env);
    load_yaml::add_functions(&mut env);
    header::add_functions(&mut env);
    escape::add_filters(&mut env);

    Ok(env)
}
