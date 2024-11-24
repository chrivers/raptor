use std::io::{Error, ErrorKind, Write};
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
use crate::sandbox::{ConsoleMode, Settings, SpawnBuilder};
use crate::{RaptorError, RaptorResult};

#[derive(Debug)]
pub struct Sandbox {
    proc: Child,
    conn: UnixStream,
    tempdir: Option<Utf8TempDir>,
    int_root: Utf8PathBuf,
    top_layer: Utf8PathBuf,
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
    let mut src = File::open(from.as_ref())?;
    let mode = src.metadata()?.permissions().mode();
    let dst = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(mode)
        .open(to.as_ref())?;

    std::io::copy(&mut src, &mut BufWriter::with_capacity(128 * 1024, dst))?;

    Ok(())
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

    pub fn new(layers: &[&Utf8Path]) -> RaptorResult<Self> {
        let tempdir = Builder::new().prefix("raptor-").tempdir()?;

        /* external root is the absolute path of the tempdir */
        let ext_root = tempdir.path();

        /* internal root is the namespace path where the tempdir will be mounted */
        let int_root = Utf8Path::new("/").join(ext_root.file_name().unwrap());

        let ext_socket_path = ext_root.join("raptor");
        let ext_client_path = ext_root.join("nspawn-client");

        let int_socket_path = int_root.join("raptor");
        let int_client_path = int_root.join("nspawn-client");

        copy_file(Self::NSPAWN_CLIENT_PATH, ext_client_path)?;

        let listen = UnixListener::bind(ext_socket_path)?;

        let mut proc = SpawnBuilder::new()
            .quiet(true)
            .sudo(true)
            .uuid(Uuid::new_v4())
            .settings(Settings::False)
            .setenv("RAPTOR_NSPAWN_SOCKET", int_socket_path.as_str())
            .root_overlays(layers)
            .bind_ro(ext_root, &int_root)
            .console(ConsoleMode::ReadOnly)
            .directory(layers[0])
            .arg(int_client_path.as_str())
            .command()
            .spawn()?;

        match Self::wait_for_startup(listen, &mut proc) {
            Ok(conn) => Ok(Self {
                proc,
                conn,
                tempdir: Some(tempdir),
                int_root,
                top_layer: layers[layers.len() - 1].into(),
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

    pub fn create_file_handle(
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
        let mount = self
            .top_layer
            .join(self.int_root.strip_prefix("/").unwrap());
        std::fs::remove_dir(mount)?;
        Ok(())
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
