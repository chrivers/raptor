use minijinja::syntax::SyntaxConfig;
use minijinja::value::Rest;
use minijinja::{Environment, ErrorKind, State, UndefinedBehavior};

use crate::RaptorResult;

#[allow(clippy::needless_pass_by_value)]
fn trace(state: &State, args: Rest<String>) {
    log::trace!("{} :: {}", state.name(), args.join(" "));
}

#[allow(clippy::needless_pass_by_value)]
fn debug(state: &State, args: Rest<String>) {
    log::debug!("{} :: {}", state.name(), args.join(" "));
}

#[allow(clippy::needless_pass_by_value)]
fn info(state: &State, args: Rest<String>) {
    log::info!("{} :: {}", state.name(), args.join(" "));
}

#[allow(clippy::needless_pass_by_value)]
fn warning(state: &State, args: Rest<String>) {
    log::warn!("{} :: {}", state.name(), args.join(" "));
}

#[allow(clippy::needless_pass_by_value)]
fn error(state: &State, args: Rest<String>) {
    log::error!("{} :: {}", state.name(), args.join(" "));
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

    env.add_function("trace", trace);
    env.add_function("debug", debug);
    env.add_function("info", info);
    env.add_function("warning", warning);
    env.add_function("error", error);

    Ok(env)
}
