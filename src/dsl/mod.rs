mod chown;
mod copy;
mod env;
mod from;
mod include;
mod inst;
mod invoke;
mod item;
mod origin;
mod program;
mod render;
mod run;
mod workdir;
mod write;

pub use chown::*;
pub use copy::*;
pub use env::*;
pub use from::*;
pub use include::*;
pub use inst::*;
pub use invoke::*;
pub use item::*;
pub use origin::*;
pub use program::*;
pub use render::*;
pub use run::*;
pub use workdir::*;
pub use write::*;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Statement {
    pub inst: Instruction,
    pub origin: Origin,
}
