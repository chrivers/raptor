use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::Child;

use camino::{Utf8Path, Utf8PathBuf};
use tempfile::{Builder, TempDir};

use crate::client::{
    Account, FramedRead, FramedWrite, Request, RequestCloseFd, RequestCreateFile, RequestRun,
    RequestWriteFd, Response,
};
use crate::dsl::Chown;
use crate::sandbox::{ConsoleMode, Settings, SpawnBuilder};
use crate::{RaptorError, RaptorResult};

pub struct Sandbox {
    proc: Child,
    conn: UnixStream,
    tempdir: TempDir,
    int_root: Utf8PathBuf,
    top_layer: Utf8PathBuf,
}

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
            tempdir,
            int_root,
            top_layer: layers[layers.len() - 1].into(),
        })
    }

    fn rpc(&mut self, req: &Request) -> RaptorResult<i32> {
        self.conn.write_framed(req)?;
        self.conn
            .read_framed::<Response>()?
            .map_err(RaptorError::RunError)
    }

    pub fn run(&mut self, cmd: &[String]) -> RaptorResult<i32> {
        self.rpc(&Request::Run(RequestRun {
            arg0: cmd[0].clone(),
            argv: cmd.to_vec(),
        }))
    }

    pub fn create_file_handle(
        &mut self,
        path: &Utf8Path,
        owner: Option<Chown>,
        mode: Option<u32>,
    ) -> RaptorResult<SandboxFile> {
        SandboxFile::new(self, path, owner, mode)
    }

    pub fn close(mut self) -> RaptorResult<()> {
        self.conn.write_framed(Request::Shutdown)?;
        self.conn.shutdown(std::net::Shutdown::Write)?;
        self.proc.wait()?;
        self.tempdir.close()?;
        let mount = self
            .top_layer
            .join(self.int_root.strip_prefix("/").unwrap());
        std::fs::remove_dir(mount)?;
        Ok(())
    }
}
