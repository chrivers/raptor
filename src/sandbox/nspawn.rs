use std::{collections::BTreeMap, process::Command};

use serde::{Deserialize, Serialize};
use serde_variant::to_variant_name;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConsoleMode {
    Interactive,
    ReadOnly,
    Passive,
    Pipe,
    Autopipe,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Settings {
    True,
    False,
    Override,
    Trusted,
}

#[must_use]
pub fn escape_colon(path: &str) -> String {
    path.replace(':', "\\:")
}

#[derive(Default)]
pub struct SpawnBuilder<'a> {
    sudo: bool,
    quiet: bool,
    args: Vec<String>,
    settings: Option<Settings>,
    directory: Option<String>,
    console: Option<ConsoleMode>,
    root_overlay: Vec<String>,
    bind: Vec<(String, String)>,
    bind_ro: Vec<(String, String)>,
    environment: BTreeMap<&'a str, &'a str>,
}

impl<'a> SpawnBuilder<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn arg(mut self, arg: &str) -> Self {
        self.args.push(arg.into());
        self
    }

    #[must_use]
    pub const fn sudo(mut self, sudo: bool) -> Self {
        self.sudo = sudo;
        self
    }

    #[must_use]
    pub const fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    #[must_use]
    pub const fn console(mut self, mode: ConsoleMode) -> Self {
        self.console = Some(mode);
        self
    }

    #[must_use]
    pub const fn settings(mut self, settings: Settings) -> Self {
        self.settings = Some(settings);
        self
    }

    #[must_use]
    pub fn setenv(mut self, key: &'a str, value: &'a str) -> Self {
        self.environment.insert(key, value);
        self
    }

    #[must_use]
    pub fn root_overlay(mut self, overlay: &str) -> Self {
        self.root_overlay.push(escape_colon(overlay));
        self
    }

    #[must_use]
    pub fn root_overlays(mut self, overlays: &[&str]) -> Self {
        for overlay in overlays {
            self.root_overlay.push(escape_colon(overlay));
        }
        self
    }

    #[must_use]
    pub fn bind(mut self, src: &str, dst: &str) -> Self {
        self.bind.push((escape_colon(src), escape_colon(dst)));
        self
    }

    #[must_use]
    pub fn bind_ro(mut self, src: &str, dst: &str) -> Self {
        self.bind_ro.push((escape_colon(src), escape_colon(dst)));
        self
    }

    #[must_use]
    pub fn directory(mut self, path: &str) -> Self {
        self.directory = Some(path.into());
        self
    }
}

impl<'a> SpawnBuilder<'a> {
    #[must_use]
    pub fn build(&self) -> Vec<String> {
        let mut res = vec![];

        if self.sudo {
            res.push("sudo".into());
        }

        res.push("systemd-nspawn".into());

        if self.quiet {
            res.push("-q".into());
        }

        if let Some(mode) = self.console {
            res.push("--console".into());
            res.push(to_variant_name(&mode).unwrap().into());
        }

        if let Some(settings) = self.settings {
            res.push("--settings".into());
            res.push(to_variant_name(&settings).unwrap().into());
        }

        if !self.root_overlay.is_empty() {
            res.push("--overlay".into());
            res.push(format!("{}:/", self.root_overlay.join(":")));
        }

        for (src, dst) in &self.bind {
            res.push("--bind".into());
            res.push(format!("{src}:{dst}"));
        }

        for (src, dst) in &self.bind_ro {
            res.push("--bind-ro".into());
            res.push(format!("{src}:{dst}"));
        }

        if let Some(dir) = &self.directory {
            res.push("-D".into());
            res.push(dir.clone());
        }

        for (name, value) in &self.environment {
            res.push("--setenv".into());
            res.push(format!("{name}={value}"));
        }

        res.extend(self.args.clone());

        res
    }

    #[must_use]
    pub fn command(&self) -> Command {
        let args = self.build();

        let mut cmd = Command::new(&args[0]);

        cmd.args(&args[1..]);

        cmd
    }
}
