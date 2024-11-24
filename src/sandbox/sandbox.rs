use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::os::unix::process::ExitStatusExt;
use std::process::{Child, ExitStatus};

use camino::{Utf8Path, Utf8PathBuf};
use nix::errno::Errno;
use tempfile::{Builder, TempDir};
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
    tempdir: Option<TempDir>,
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
            Err(err) => Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, err)),
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

impl Sandbox {
    pub fn new(layers: &[&Utf8Path]) -> RaptorResult<Self> {
        let tempdir = Builder::new().prefix("raptor-").tempdir()?;

        let ext_root = Utf8PathBuf::from_path_buf(tempdir.path().to_path_buf())?;
        let ext_socket_path = ext_root.join("raptor");
        let ext_client_path = ext_root.join("nspawn-client");

        let int_root = Utf8PathBuf::from("/").join(ext_root.file_name().unwrap());

        let int_socket_path = int_root.join("raptor");
        let int_client_path = int_root.join("nspawn-client");

        std::fs::copy(
            "target/x86_64-unknown-linux-musl/release/nspawn-client",
            ext_client_path,
        )?;

        let listen = UnixListener::bind(ext_socket_path)?;

        let proc = SpawnBuilder::new()
            .quiet(true)
            .sudo(true)
            .uuid(Uuid::new_v4())
            .settings(Settings::False)
            .setenv("RAPTOR_NSPAWN_SOCKET", int_socket_path.as_str())
            .root_overlays(layers)
            .bind_ro(&ext_root, &int_root)
            .console(ConsoleMode::ReadOnly)
            .directory(layers[0])
            .arg(int_client_path.as_str())
            .command()
            .spawn()?;

        let conn = listen.accept()?.0;

        Ok(Self {
            proc,
            conn,
            tempdir: Some(tempdir),
            int_root,
            top_layer: layers[layers.len() - 1].into(),
        })
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
