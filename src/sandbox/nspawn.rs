use std::process::Command;
use std::{collections::BTreeMap, fmt::Display};

use camino::{Utf8Path, Utf8PathBuf};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_variant::to_variant_name;
use uuid::Uuid;

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

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkJournal {
    No,
    Host,
    TryHost,
    Guest,
    TryGuest,
    Auto,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResolvConf {
    Off,
    CopyHost,
    CopyStatic,
    CopyUplink,
    CopyStub,
    ReplaceHost,
    ReplaceStatic,
    ReplaceUplink,
    ReplaceStub,
    BindHost,
    BindStatic,
    BindUplink,
    BindStub,
    Delete,
    Auto,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Timezone {
    Off,
    Copy,
    Bind,
    Symlink,
    Delete,
    Auto,
}

#[must_use]
pub fn escape_colon(path: &Utf8Path) -> String {
    path.as_str().replace(':', "\\:")
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum BindMode {
    #[default]
    NoIdmap,
    Idmap,
    RootIdmap,
    OwnerIdmap,
}

impl BindMode {
    const DEFAULT: Self = Self::NoIdmap;
}

#[derive(Clone, Debug)]
pub struct BindMount {
    src: Utf8PathBuf,
    dst: Utf8PathBuf,
    mode: BindMode,
}

impl Display for BindMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoIdmap => write!(f, "noidmap"),
            Self::Idmap => write!(f, "idmap"),
            Self::RootIdmap => write!(f, "rootidmap"),
            Self::OwnerIdmap => write!(f, "owneridmap"),
        }
    }
}

impl Display for BindMount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &escape_colon(&self.src))?;
        if self.mode != BindMode::DEFAULT || self.src != self.dst {
            write!(f, ":{}", &escape_colon(&self.dst))?;
        }
        if self.mode != BindMode::DEFAULT {
            write!(f, ":{}", self.mode)?;
        }
        Ok(())
    }
}

impl BindMount {
    pub fn new(src: impl AsRef<Utf8Path>, dst: impl AsRef<Utf8Path>) -> Self {
        Self {
            src: src.as_ref().to_path_buf(),
            dst: dst.as_ref().to_path_buf(),
            mode: BindMode::DEFAULT,
        }
    }

    #[must_use]
    pub fn with_mode(self, mode: BindMode) -> Self {
        Self { mode, ..self }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SpawnBuilder {
    sudo: bool,
    quiet: bool,
    suppress_sync: bool,
    args: Vec<String>,
    uuid: Option<Uuid>,
    settings: Option<Settings>,
    console: Option<ConsoleMode>,
    link_journal: Option<LinkJournal>,
    resolv_conf: Option<ResolvConf>,
    timezone: Option<Timezone>,
    directory: Option<Utf8PathBuf>,
    root_overlay: Vec<Utf8PathBuf>,
    overlay: Vec<Vec<Utf8PathBuf>>,
    overlay_ro: Vec<Vec<Utf8PathBuf>>,
    bind: Vec<BindMount>,
    bind_ro: Vec<BindMount>,
    environment: BTreeMap<String, String>,
}

impl SpawnBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    #[must_use]
    pub fn args(mut self, args: &[impl AsRef<str>]) -> Self {
        self.args
            .extend(args.iter().map(AsRef::as_ref).map(ToString::to_string));
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
    pub const fn suppress_sync(mut self, suppress_sync: bool) -> Self {
        self.suppress_sync = suppress_sync;
        self
    }

    #[must_use]
    pub const fn uuid(mut self, uuid: Uuid) -> Self {
        self.uuid = Some(uuid);
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
    pub const fn link_journal(mut self, link_journal: LinkJournal) -> Self {
        self.link_journal = Some(link_journal);
        self
    }

    #[must_use]
    pub const fn resolv_conf(mut self, resolv_conf: ResolvConf) -> Self {
        self.resolv_conf = Some(resolv_conf);
        self
    }

    #[must_use]
    pub const fn timezone(mut self, timezone: Timezone) -> Self {
        self.timezone = Some(timezone);
        self
    }

    #[must_use]
    pub fn setenv(mut self, key: &str, value: &str) -> Self {
        self.environment.insert(key.to_string(), value.to_string());
        self
    }

    #[must_use]
    pub fn root_overlay(mut self, overlay: impl Into<Utf8PathBuf>) -> Self {
        self.root_overlay.push(overlay.into());
        self
    }

    #[must_use]
    pub fn root_overlays(mut self, overlays: &[impl AsRef<Utf8Path>]) -> Self {
        self.root_overlay.extend(
            overlays
                .iter()
                .map(AsRef::as_ref)
                .map(Utf8Path::to_path_buf),
        );
        self
    }

    #[must_use]
    pub fn bind(mut self, mount: impl Into<BindMount>) -> Self {
        self.bind.push(mount.into());
        self
    }

    #[must_use]
    pub fn bind_ro(mut self, mount: impl Into<BindMount>) -> Self {
        self.bind_ro.push(mount.into());
        self
    }

    #[must_use]
    pub fn directory(mut self, path: &Utf8Path) -> Self {
        self.directory = Some(path.to_path_buf());
        self
    }

    #[must_use]
    pub fn overlay(mut self, source: &[impl AsRef<Utf8Path>], dest: impl AsRef<Utf8Path>) -> Self {
        let mut layers: Vec<Utf8PathBuf> = source
            .iter()
            .map(AsRef::as_ref)
            .map(Utf8Path::to_path_buf)
            .collect();
        layers.push(dest.as_ref().to_path_buf());

        self.overlay.push(layers);
        self
    }

    #[must_use]
    pub fn overlay_ro(
        mut self,
        source: &[impl AsRef<Utf8Path>],
        dest: impl AsRef<Utf8Path>,
    ) -> Self {
        let mut layers: Vec<Utf8PathBuf> = source
            .iter()
            .map(AsRef::as_ref)
            .map(Utf8Path::to_path_buf)
            .collect();
        layers.push(dest.as_ref().to_path_buf());

        self.overlay_ro.push(layers);
        self
    }
}

impl SpawnBuilder {
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

        if self.suppress_sync {
            res.push("--suppress-sync=true".into());
        }

        if let Some(mode) = self.console {
            res.push("--console".into());
            res.push(to_variant_name(&mode).unwrap().into());
        }

        if let Some(settings) = self.settings {
            res.push("--settings".into());
            res.push(to_variant_name(&settings).unwrap().into());
        }

        if let Some(link_journal) = self.link_journal {
            res.push("--link-journal".into());
            res.push(to_variant_name(&link_journal).unwrap().into());
        }

        if let Some(resolv_conf) = self.resolv_conf {
            res.push("--resolv-conf".into());
            res.push(to_variant_name(&resolv_conf).unwrap().into());
        }

        if let Some(timezone) = self.timezone {
            res.push("--timezone".into());
            res.push(to_variant_name(&timezone).unwrap().into());
        }

        if !self.root_overlay.is_empty() {
            res.push("--overlay".into());

            let mut overlays = self
                .root_overlay
                .clone()
                .iter()
                .map(AsRef::as_ref)
                .map(escape_colon)
                .collect::<Vec<_>>();

            overlays.push("/".into());

            res.push(overlays.join(":"));
        }

        for overlay in &self.overlay {
            res.push("--overlay".into());

            let overlays = overlay
                .iter()
                .map(Utf8PathBuf::as_path)
                .map(escape_colon)
                .join(":");

            res.push(overlays);
        }

        for overlay in &self.overlay_ro {
            res.push("--overlay-ro".into());

            let overlays = overlay
                .iter()
                .map(Utf8PathBuf::as_path)
                .map(escape_colon)
                .join(":");

            res.push(overlays);
        }

        for mount in &self.bind {
            res.push("--bind".into());
            res.push(mount.to_string());
        }

        for mount in &self.bind_ro {
            res.push("--bind-ro".into());
            res.push(mount.to_string());
        }

        let uuid = &self.uuid.unwrap_or_else(Uuid::new_v4);
        res.push("--machine".into());
        res.push(format!("raptor-{uuid}"));

        res.push("--uuid".into());
        res.push(format!("{uuid}"));

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

#[cfg(test)]
mod tests {
    use maplit::hashmap;
    use raptor_parser::ast::{InstMount, MountOptions, MountType};

    use crate::build::RaptorBuilder;
    use crate::program::Loader;
    use crate::runner::AddMounts;
    use crate::sandbox::{Sandbox, SpawnBuilder};
    use crate::{RaptorError, RaptorResult};

    #[test]
    fn add_mounts() -> RaptorResult<()> {
        let loader = Loader::new()?;
        let builder = RaptorBuilder::new(loader, Sandbox::find_falcon_dev()?, false);

        let mut mount = InstMount {
            opts: MountOptions {
                mtype: MountType::File,
                readonly: false,
                optional: false,
            },
            name: "foo1".into(),
            dest: "bar".into(),
        };

        let mounts1 = hashmap! { "foo1" => vec!["foo-src"] };
        let mounts2 = hashmap! { "foo2" => vec!["foo-src"] };

        // required mount (exists)
        let sb = SpawnBuilder::new().add_mounts(&[&mount], &builder, &mounts1, "/tmp")?;
        assert_eq!(sb.bind.len(), 1);

        // required mount (missing)
        let sb = SpawnBuilder::new().add_mounts(&[&mount], &builder, &mounts2, "/tmp");
        assert!(matches!(sb.unwrap_err(), RaptorError::MountMissing(_)));

        // optional mounts below
        mount.opts.optional = true;

        // optional mount (exists)
        let sb = SpawnBuilder::new().add_mounts(&[&mount], &builder, &mounts1, "/tmp")?;
        assert_eq!(sb.bind.len(), 1);

        // optional mount (missing)
        let sb = SpawnBuilder::new().add_mounts(&[&mount], &builder, &mounts2, "/tmp")?;
        assert_eq!(sb.bind.len(), 0);

        Ok(())
    }

    #[test]
    fn add_mounts_ro() -> RaptorResult<()> {
        let loader = Loader::new()?;
        let builder = RaptorBuilder::new(loader, Sandbox::find_falcon_dev()?, false);

        let mut mount = InstMount {
            opts: MountOptions {
                mtype: MountType::File,
                readonly: true,
                optional: false,
            },
            name: "foo1".into(),
            dest: "bar".into(),
        };

        let mounts1 = hashmap! { "foo1" => vec!["foo-src"] };
        let mounts2 = hashmap! { "foo2" => vec!["foo-src"] };

        // required mount (exists)
        let sb = SpawnBuilder::new().add_mounts(&[&mount], &builder, &mounts1, "/tmp")?;
        assert_eq!(sb.bind_ro.len(), 1);

        // required mount (missing)
        let sb = SpawnBuilder::new().add_mounts(&[&mount], &builder, &mounts2, "/tmp");
        assert!(matches!(sb.unwrap_err(), RaptorError::MountMissing(_)));

        // optional mounts below
        mount.opts.optional = true;

        // optional mount (exists)
        let sb = SpawnBuilder::new().add_mounts(&[&mount], &builder, &mounts1, "/tmp")?;
        assert_eq!(sb.bind_ro.len(), 1);

        // optional mount (missing)
        let sb = SpawnBuilder::new().add_mounts(&[&mount], &builder, &mounts2, "/tmp")?;
        assert_eq!(sb.bind_ro.len(), 0);

        Ok(())
    }
}
