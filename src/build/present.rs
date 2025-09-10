use colored::Colorize;
use itertools::Itertools;

use crate::build::{BuildTarget, BuildTargetStats};
use crate::dsl::Item;
use crate::RaptorResult;

pub struct Presenter<'a>(&'a BuildTargetStats);

impl<'a> Presenter<'a> {
    #[must_use]
    pub const fn new(stats: &'a BuildTargetStats) -> Self {
        Self(stats)
    }

    pub fn present_program(&self, name: &str, indent: usize) -> RaptorResult<()> {
        let prefix = " | ".repeat(indent);

        if let BuildTarget::Program(program) = &self.0.targets[name] {
            for inst in &program.code {
                match inst {
                    Item::Statement(stmt) => {
                        println!("{prefix}{}", stmt.inst);
                    }

                    Item::Program(_program) => {
                        println!("{prefix}");
                    }
                }
            }
        }

        if let Some(rmap) = self.0.rmap.get(name) {
            for sub in rmap.iter().sorted() {
                println!("{prefix}");
                println!("{prefix}{} [{}]", "# target".dimmed(), sub.bright_white());
                println!(
                    "{prefix}[{}] -> [{}]",
                    name.bright_white(),
                    sub.bright_white()
                );
                self.present_program(sub, indent + 1)?;
            }
        }

        Ok(())
    }

    pub fn present(&self) -> RaptorResult<()> {
        for root in self.0.roots.iter().sorted() {
            println!("ROOT {root}");
            self.present_program(root, 0)?;
            println!();
        }

        Ok(())
    }
}
