use std::{collections::BTreeMap, process::Command};

use camino::Utf8Path;
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
pub fn escape_colon(path: &Utf8Path) -> String {
    path.as_str().replace(':', "\\:")
}

#[derive(Default)]
pub struct SpawnBuilder<'a> {
    sudo: bool,
    quiet: bool,
    args: Vec<&'a str>,
    settings: Option<Settings>,
    console: Option<ConsoleMode>,
    directory: Option<&'a Utf8Path>,
    root_overlay: Vec<&'a Utf8Path>,
    bind: Vec<(&'a Utf8Path, &'a Utf8Path)>,
    bind_ro: Vec<(&'a Utf8Path, &'a Utf8Path)>,
    environment: BTreeMap<&'a str, &'a str>,
}

impl<'a> SpawnBuilder<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn arg(mut self, arg: &'a str) -> Self {
        self.args.push(arg);
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
    pub fn root_overlay(mut self, overlay: &'a Utf8Path) -> Self {
        self.root_overlay.push(overlay);
        self
    }

    #[must_use]
    pub fn root_overlays(mut self, overlays: &[&'a Utf8Path]) -> Self {
        self.root_overlay.extend(overlays);
        self
    }

    #[must_use]
    pub fn bind(mut self, src: &'a Utf8Path, dst: &'a Utf8Path) -> Self {
        self.bind.push((src, dst));
        self
    }

    #[must_use]
    pub fn bind_ro(mut self, src: &'a Utf8Path, dst: &'a Utf8Path) -> Self {
        self.bind_ro.push((src, dst));
        self
    }

    #[must_use]
    pub const fn directory(mut self, path: &'a Utf8Path) -> Self {
        self.directory = Some(path);
        self
    }
}

impl SpawnBuilder<'_> {
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

            let mut overlays = self
                .root_overlay
                .clone()
                .into_iter()
                .map(escape_colon)
                .collect::<Vec<_>>();

            overlays.push("/".into());

            res.push(overlays.join(":"));
        }

        for (src, dst) in &self.bind {
            res.push("--bind".into());
            res.push(format!("{}:{}", escape_colon(src), escape_colon(dst)));
        }

        for (src, dst) in &self.bind_ro {
            res.push("--bind-ro".into());
            res.push(format!("{}:{}", escape_colon(src), escape_colon(dst)));
        }

        if let Some(dir) = &self.directory {
            res.push("-D".into());
            res.push((*dir).to_string());
        }

        for (name, value) in &self.environment {
            res.push("--setenv".into());
            res.push(format!("{name}={value}"));
        }

        res.extend(self.args.iter().map(ToString::to_string));

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
