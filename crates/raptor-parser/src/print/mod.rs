use camino::Utf8Path;
use colored::Colorize;

use std::fmt::{Debug, Formatter, Result};

use crate::ast::{Chown, FromSource, IncludeArg, InstEnvAssign};

pub trait Theme {
    fn keyword(&mut self, name: &str) -> Result;
    fn chmod(&mut self, chmod: &Option<u32>) -> Result;
    fn chown(&mut self, chown: &Option<Chown>) -> Result;
    fn from(&mut self, src: &FromSource) -> Result;
    fn src(&mut self, src: &Utf8Path) -> Result;
    fn dest(&mut self, dest: &Utf8Path) -> Result;
    fn include_arg(&mut self, arg: &IncludeArg) -> Result;
    fn env_arg(&mut self, arg: &InstEnvAssign) -> Result;
    fn name(&mut self, name: &str) -> Result;
    fn value(&mut self, value: impl Debug) -> Result;
}

impl Theme for Formatter<'_> {
    fn keyword(&mut self, name: &str) -> Result {
        write!(self, "{}", name.bright_blue())
    }

    fn chmod(&mut self, chmod: &Option<u32>) -> Result {
        if let Some(chmod) = chmod {
            write!(
                self,
                " {} {}",
                "--chmod".bright_white(),
                format!("{chmod:04o}").cyan()
            )?;
        }
        Ok(())
    }

    fn chown(&mut self, chown: &Option<Chown>) -> Result {
        if let Some(chown) = chown {
            write!(
                self,
                " {} {}",
                "--chown".bright_white(),
                format!("{chown}").cyan()
            )?;
        }
        Ok(())
    }

    fn from(&mut self, src: &FromSource) -> Result {
        write!(self, " {}", format!("{src}").green())
    }

    fn src(&mut self, src: &Utf8Path) -> Result {
        write!(self, " {}", format!("{src:?}").green())
    }

    fn dest(&mut self, dest: &Utf8Path) -> Result {
        write!(self, " {}", format!("{dest:?}").bright_green())
    }

    fn include_arg(&mut self, arg: &IncludeArg) -> Result {
        write!(
            self,
            " {}{}{}",
            arg.name.yellow(),
            "=".dimmed(),
            format!("{}", arg.value).red()
        )
    }

    fn env_arg(&mut self, arg: &InstEnvAssign) -> Result {
        write!(
            self,
            " {}{}{}",
            arg.key.yellow(),
            "=".dimmed(),
            format!("{:?}", arg.value).red()
        )
    }

    fn name(&mut self, name: &str) -> Result {
        write!(self, " {}", name.yellow())
    }

    fn value(&mut self, value: impl Debug) -> Result {
        write!(self, " {}", format!("{value:?}").bright_red())
    }
}
