use std::fmt::Display;

use colored::Colorize;

use crate::print::Theme;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum MountType {
    Simple,
    Layers,
    Overlay,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MountOptions {
    pub mtype: MountType,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InstMount {
    pub opts: MountOptions,
    pub name: String,
    pub dest: String,
}

impl Display for InstMount {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.keyword("MOUNT")?;
        write!(f, "{}", &self.opts)?;
        f.name(&self.name)?;
        f.dest(&self.dest)
    }
}

impl Display for MountOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " {}", self.mtype)
    }
}

impl Display for MountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Simple => "--simple",
            Self::Layers => "--layers",
            Self::Overlay => "--overlay",
        };

        write!(f, "{}", name.bright_white())
    }
}
