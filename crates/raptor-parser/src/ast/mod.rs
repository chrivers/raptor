mod chown;
mod cmd;
mod copy;
mod entrypoint;
mod env;
mod from;
mod include;
mod inst;
mod invoke;
mod mkdir;
mod mount;
mod origin;
mod render;
mod run;
mod workdir;
mod write;

pub use chown::*;
pub use cmd::*;
pub use copy::*;
pub use entrypoint::*;
pub use env::*;
pub use from::*;
pub use include::*;
pub use inst::*;
pub use invoke::*;
pub use mkdir::*;
pub use mount::*;
pub use origin::*;
pub use render::*;
pub use run::*;
pub use workdir::*;
pub use write::*;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Statement {
    pub inst: Instruction,
    pub origin: Origin,
}
