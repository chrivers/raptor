use minijinja::value::Rest;
use minijinja::{Environment, State};

#[allow(clippy::needless_pass_by_value)]
pub fn trace(state: &State, args: Rest<String>) {
    trace!("{} | {}", state.name(), args.join(" "));
}

#[allow(clippy::needless_pass_by_value)]
pub fn debug(state: &State, args: Rest<String>) {
    debug!("{} | {}", state.name(), args.join(" "));
}

#[allow(clippy::needless_pass_by_value)]
pub fn info(state: &State, args: Rest<String>) {
    info!("{} | {}", state.name(), args.join(" "));
}

#[allow(clippy::needless_pass_by_value)]
pub fn warning(state: &State, args: Rest<String>) {
    warn!("{} | {}", state.name(), args.join(" "));
}

#[allow(clippy::needless_pass_by_value)]
pub fn error(state: &State, args: Rest<String>) {
    error!("{} | {}", state.name(), args.join(" "));
}

pub fn add_functions(env: &mut Environment) {
    env.add_function("trace", trace);
    env.add_function("debug", debug);
    env.add_function("info", info);
    env.add_function("warning", warning);
    env.add_function("error", error);
}
