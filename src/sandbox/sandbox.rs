use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::os::unix::process::ExitStatusExt;
use std::process::{Child, ExitStatus};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use camino::{Utf8Path, Utf8PathBuf};
use camino_tempfile::{Builder, Utf8TempDir};
use nix::errno::Errno;
use uuid::Uuid;

use crate::client::{
    Account, FramedRead, FramedWrite, Request, RequestChangeDir, RequestCloseFd, RequestCreateFile,
    RequestRun, RequestSetEnv, RequestWriteFd, Response,
};
use crate::dsl::Chown;
use crate::sandbox::{ConsoleMode, LinkJournal, Settings, SpawnBuilder};
use crate::util::io_fast_copy;
use crate::{RaptorError, RaptorResult};

#[derive(Debug)]
pub struct Sandbox {
    proc: Child,
    conn: UnixStream,
    mount: Option<Utf8PathBuf>,
    tempdir: Option<Utf8TempDir>,
}

#[derive(Debug)]
pub struct SandboxFile<'sb> {
    sandbox: &'sb mut Sandbox,
    fd: i32,
}

impl<'sb> SandboxFile<'sb> {
    pub fn new(
        sandbox: &'sb mut Sandbox,
        path: &Utf8Path,
        owner: Option<Chown>,
        mode: Option<u32>,
    ) -> RaptorResult<Self> {
        let Chown { user, group } = owner.unwrap_or_default();
        let fd = sandbox.rpc(&Request::CreateFile(RequestCreateFile {
            path: path.to_owned(),
            user: user.map(Account::Name),
            group: group.map(Account::Name),
            mode,
        }))?;
        Ok(Self { sandbox, fd })
    }
}

impl Write for SandboxFile<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.sandbox.rpc(&Request::WriteFd(RequestWriteFd {
            fd: self.fd,
            data: buf.to_vec(),
        })) {
            Ok(_) => Ok(buf.len()),
            Err(RaptorError::IoError(err)) => Err(err),
            Err(err) => Err(Error::new(ErrorKind::BrokenPipe, err)),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for SandboxFile<'_> {
    fn drop(&mut self) {
        let _ = self
            .sandbox
            .rpc(&Request::CloseFd(RequestCloseFd { fd: self.fd }));
    }
}

fn copy_file(from: impl AsRef<Utf8Path>, to: impl AsRef<Utf8Path>) -> RaptorResult<()> {
    let src = File::open(from.as_ref())?;
    let mode = src.metadata()?.permissions().mode();
    let dst = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(mode)
        .open(to.as_ref())?;

    io_fast_copy(src, dst)
}

impl Sandbox {
    const START_TIMEOUT: Duration = Duration::from_secs(2);
    const CHECK_TIMEOUT: Duration = Duration::from_millis(100);

    /* TODO: ugly hack, but works for testing */
    const NSPAWN_CLIENT_PATH: &str = "target/x86_64-unknown-linux-musl/release/nspawn-client";

    fn wait_for_startup(listen: UnixListener, proc: &mut Child) -> RaptorResult<UnixStream> {
        let (tx, rx) = mpsc::channel();

        /* Spawn a thread that waits for the sandbox to start up, and the
         * nspawn-client to connect from inside the namespace */
        std::thread::spawn(move || -> RaptorResult<_> { Ok(tx.send(listen.accept()?.0)?) });

        /* Loop until START_TIMEOUT is reached, checking the sandbox process
         * every time CHECK_TIMEOUT has passed */
        for _ in 0..(Self::START_TIMEOUT.as_millis() / Self::CHECK_TIMEOUT.as_millis()) {
            if let Some(status) = proc.try_wait()? {
                return Err(RaptorError::SandboxRunError(status));
            }

            match rx.recv_timeout(Self::CHECK_TIMEOUT) {
                Ok(conn) => return Ok(conn),
                Err(RecvTimeoutError::Timeout) => continue,
                Err(err) => Err(err)?,
            };
        }

        Err(RaptorError::SandboxRequestError(Errno::ECONNABORTED))
    }

    pub fn new(layers: &[&Utf8Path], rootdir: &Utf8Path) -> RaptorResult<Self> {
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

        This dir with an ampty `/usr` dir, is the `tempdir`.

        The `conndir` serves an actual purpose. It contains a copy of the raptor
        `nspawn-client` binary, as well as the unix socket that the
        `nspawn-client` will connect to. This directory is then bind-mounted
        into the container.

        temp:
          - /usr (<-- empty dir)

        conn:
          - /raptor (<-- socket)
          - /nspawn-client (<-- client binary)

         */
        let tempdir = Builder::new().prefix("raptor-temp-").tempdir()?;
        let conndir = Builder::new().prefix("raptor-conn-").tempdir()?;

        let uuid = Uuid::new_v4();
        let uuid_name = uuid.as_simple().to_string();

        /* ensure the build directory exists before we start the build (with
         * sudo). This ensures the build root is owned by the current user, thus
         * allowing our cleanup rmdir() to succeed */
        std::fs::create_dir_all(rootdir)?;

        /* the ephemeral root directory needs to have /usr for systemd-nspawn to accept it */
        std::fs::create_dir(tempdir.path().join("usr"))?;

        /* external root is the absolute path of the tempdir */
        let ext_root = conndir.path();

        /* internal root is the namespace path where the tempdir will be mounted */
        let int_root = Utf8PathBuf::from(format!("/raptor-{uuid_name}"));

        let ext_socket_path = ext_root.join("raptor");
        let ext_client_path = ext_root.join("nspawn-client");

        let int_socket_path = int_root.join("raptor");
        let int_client_path = int_root.join("nspawn-client");

        copy_file(Self::NSPAWN_CLIENT_PATH, ext_client_path)?;

        let listen = UnixListener::bind(ext_socket_path)?;

        let spawn = SpawnBuilder::new()
            .quiet(true)
            .sudo(true)
            .uuid(uuid)
            .link_journal(LinkJournal::No)
            .settings(Settings::False)
            .setenv("RAPTOR_NSPAWN_SOCKET", int_socket_path.as_str())
            .root_overlays(layers)
            .root_overlay(rootdir)
            .bind_ro(ext_root, &int_root)
            .console(ConsoleMode::ReadOnly)
            .directory(tempdir.path())
            .arg(int_client_path.as_str());

        debug!("Starting sandbox: {:?}", spawn.build().join(" "));

        let mut proc = spawn.command().spawn()?;

        match Self::wait_for_startup(listen, &mut proc) {
            Ok(conn) => Ok(Self {
                proc,
                conn,
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

    fn rpc(&mut self, req: &Request) -> RaptorResult<i32> {
        self.conn.write_framed(req)?;
        self.conn
            .read_framed::<Response>()?
            .map_err(|errno| RaptorError::SandboxRequestError(Errno::from_raw(errno)))
    }

    pub fn run(&mut self, cmd: &[String]) -> RaptorResult<()> {
        match self.rpc(&Request::Run(RequestRun {
            arg0: cmd[0].clone(),
            argv: cmd.to_vec(),
        })) {
            Ok(0) => Ok(()),
            Ok(n) => Err(RaptorError::SandboxRunError(ExitStatus::from_raw(n))),
            Err(err) => Err(err),
        }
    }

    pub fn create_file(
        &mut self,
        path: &Utf8Path,
        owner: Option<Chown>,
        mode: Option<u32>,
    ) -> RaptorResult<SandboxFile> {
        SandboxFile::new(self, path, owner, mode)
    }

    pub fn chdir(&mut self, dir: &str) -> RaptorResult<()> {
        self.rpc(&Request::ChangeDir(RequestChangeDir {
            cd: dir.to_string(),
        }))?;
        Ok(())
    }

    pub fn setenv(&mut self, key: &str, value: &str) -> RaptorResult<()> {
        self.rpc(&Request::SetEnv(RequestSetEnv {
            key: key.to_string(),
            value: value.to_string(),
        }))?;
        Ok(())
    }

    pub fn close(&mut self) -> RaptorResult<()> {
        self.conn.write_framed(Request::Shutdown)?;
        self.conn.shutdown(std::net::Shutdown::Write)?;
        self.proc.wait()?;
        if let Some(tempdir) = self.tempdir.take() {
            tempdir.close()?;
        }
        if let Some(mount) = self.mount.take() {
            std::fs::remove_dir(mount)?;
        }
        Ok(())
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
            let _ = self.conn.write_framed(Request::Shutdown);
            let _ = self.conn.shutdown(std::net::Shutdown::Write);
            let _ = self.proc.wait();
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
