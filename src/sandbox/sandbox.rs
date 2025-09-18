use std::os::unix::net::UnixListener;
use std::process::Stdio;
use std::thread;

use camino::{Utf8Path, Utf8PathBuf};
use camino_tempfile::{Builder, Utf8TempDir};
use uuid::Uuid;

use crate::RaptorResult;
use crate::sandbox::{
    BindMount, ConsoleMode, FalconClient, LinkJournal, ResolvConf, Settings, SpawnBuilder, Timezone,
};
use crate::util::link_or_copy_file;

#[derive(Debug)]
pub struct Sandbox {
    client: FalconClient,
    rootdir: Utf8PathBuf,
    mount: Option<Utf8PathBuf>,
    tempdir: Option<Utf8TempDir>,
}

impl Sandbox {
    /* TODO: ugly hack, but works for testing */
    pub const FALCON_PATH: &str = "target/x86_64-unknown-linux-musl/release/falcon";

    #[must_use]
    pub fn builder() -> SpawnBuilder {
        SpawnBuilder::new()
            .quiet(true)
            .suppress_sync(true)
            .link_journal(LinkJournal::No)
            .resolv_conf(ResolvConf::Off)
            .timezone(Timezone::Off)
            .settings(Settings::False)
            .console(ConsoleMode::ReadOnly)
    }

    pub fn new(layers: &[impl AsRef<Utf8Path>], rootdir: &Utf8Path) -> RaptorResult<Self> {
        Self::custom(Self::builder(), layers, rootdir)
    }

    pub fn custom(
        mut spawn: SpawnBuilder,
        layers: &[impl AsRef<Utf8Path>],
        rootdir: &Utf8Path,
    ) -> RaptorResult<Self> {
        /*
        For the sandbox, we need two directories, "temp" and "conn".

        The whole stack of layers is mounted as overlayfs "lowerdirs", with the
        rootdir as the "upperdir" (see overlayfs man page).

        We tell systemd-nspawn to mount this stack on "/", i.e. the root
        directory of the container, but it still requires us to specify a
        directory to mount as the root.

        This directory ends up being unused, but is still required.

        Even sillier, this directory is checked for the existence of `/usr`, so
        we have to create it, to make sure systemd-nspawn is happy, so it can go
        on to ignore it completely.

        This dir with an empty `/usr` dir, is the `tempdir`.

        Incidentally, systemd-nspawn also "helpfully" pre-populates our target
        directory with a number of directories (`SYSTEMD_NSPAWN_BASE_DIRS`). To
        avoid having these directories show up in all build products, we create
        all of them in `tempdir`.

        The `conndir` serves an actual purpose. It contains a copy of the raptor
        `falcon` binary, as well as the unix socket that the
        `falcon` will connect to. This directory is then bind-mounted
        into the container.

        temp:
          - /usr (<-- empty dir)

        conn:
          - /raptor (<-- socket)
          - /falcon (<-- client binary)

          | external path         | internal path  | note                            |
          |-----------------------|----------------|---------------------------------|
          | $TMP/raptor-temp-{id} | /              | 1. contains /usr                |
          |                       |                | 2. is hidden by root overlay    |
          |                       |                |                                 |
          | $TMP/raptor-conn-{id} | /raptor-{uuid} | 1. has falcon and socket |
          |                       |                | 2. is mounted read-only         |

         */
        let tempdir = Builder::new().prefix("raptor-temp-").tempdir()?;
        let conndir = Builder::new().prefix("raptor-conn-").tempdir()?;

        let uuid = Uuid::new_v4();
        let uuid_name = uuid.as_simple().to_string();

        /* the ephemeral root directory needs to have /usr for systemd-nspawn to accept it */
        let root = tempdir.path().join("root");
        std::fs::create_dir_all(root.join("usr"))?;

        /* external root is the absolute path of the tempdir */
        let ext_root = conndir.path();

        /* internal root is the namespace path where the conndir will be mounted */
        let int_root = Utf8PathBuf::from(format!("/raptor-{uuid_name}"));

        /* external name of socket and falcon client */
        let ext_socket_path = ext_root.join("raptor");
        let ext_client_path = ext_root.join("falcon");

        /* internal name of socket and falcon client */
        let int_socket_path = int_root.join("raptor");
        let int_client_path = int_root.join("falcon");

        link_or_copy_file(Self::FALCON_PATH, &ext_client_path)?;

        let listen = UnixListener::bind(ext_socket_path)?;

        spawn = spawn
            .uuid(uuid)
            .root_overlay(tempdir.path())
            .root_overlays(layers)
            .root_overlay(rootdir)
            .bind_ro(BindMount::new(ext_root, &int_root))
            .directory(&root)
            .setenv("FALCON_SOCKET", int_socket_path.as_str())
            .arg(int_client_path.as_str());

        debug!(
            "Starting sandbox: {}",
            spawn.build().join(" ").replace(" --", "\n  --")
        );

        let mut proc = spawn
            .command()
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdout = proc.stdout.take().unwrap();
        let mut stderr = proc.stderr.take().unwrap();
        thread::spawn(move || std::io::copy(&mut stdout, &mut std::io::stdout()));
        thread::spawn(move || std::io::copy(&mut stderr, &mut std::io::stderr()));

        match FalconClient::wait_for_startup(listen, &mut proc) {
            Ok(conn) => Ok(Self {
                client: FalconClient::new(proc, conn),
                rootdir: rootdir.into(),
                mount: Some(rootdir.join(int_root.strip_prefix("/").unwrap())),
                tempdir: Some(tempdir),
            }),
            Err(err) => {
                /* if we arrive here, the sandbox did not start within the timeout, so
                 * kill the half-started container and report the error */
                proc.kill()?;
                proc.wait()?;
                Err(err)
            }
        }
    }

    pub fn close(&mut self) -> RaptorResult<()> {
        if let Some(tempdir) = self.tempdir.take() {
            tempdir.close()?;
        }
        if let Some(mount) = self.mount.take() {
            std::fs::remove_dir(mount)?;
        }
        Ok(())
    }

    pub const fn client(&mut self) -> &mut FalconClient {
        &mut self.client
    }

    #[must_use]
    pub fn get_root_dir(&self) -> &Utf8Path {
        &self.rootdir
    }

    #[must_use]
    pub fn get_mount_dir(&self) -> Option<&Utf8Path> {
        self.mount.as_deref()
    }

    #[must_use]
    pub fn get_temp_dir(&self) -> Option<&Utf8Path> {
        self.tempdir.as_ref().map(Utf8TempDir::path)
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        if let Some(mount) = &self.mount {
            let _ = self.client.close();
            let _ = std::fs::remove_dir(mount);
            if let Some(tempdir) = self.tempdir.take() {
                let _ = tempdir.close();
            }
            if let Some(mount) = self.mount.take() {
                let _ = std::fs::remove_dir(mount);
            }
        }
    }
}
