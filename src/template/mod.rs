use minijinja::syntax::SyntaxConfig;
use minijinja::value::ViaDeserialize;
use minijinja::{Environment, ErrorKind, UndefinedBehavior};

use crate::RaptorResult;

#[allow(clippy::needless_pass_by_value)]
fn hello(arg: ViaDeserialize<String>) {
    log::info!("Hello {:?}", *arg);
}

pub fn make_environment<'a>() -> RaptorResult<Environment<'a>> {
    let mut env = Environment::new();
    env.set_debug(true);
    env.set_undefined_behavior(UndefinedBehavior::Strict);

    env.set_loader(|name| {
        Ok(Some(std::fs::read_to_string(name).map_err(|e| {
            minijinja::Error::new(
                ErrorKind::BadInclude,
                format!("Could not open [{name}]: {e}"),
            )
        })?))
    });

    env.set_syntax(
        SyntaxConfig::builder()
            .line_comment_prefix("# ")
            .line_statement_prefix("$ ")
            .build()?,
    );

    env.add_function("hello", hello);

    Ok(env)
}
