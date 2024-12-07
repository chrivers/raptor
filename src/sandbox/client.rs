use std::os::unix::net::{UnixListener, UnixStream};
use std::os::unix::process::ExitStatusExt;
use std::process::{Child, ExitStatus};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use camino::Utf8Path;
use nix::errno::Errno;

use crate::client::{
    FramedRead, FramedWrite, Request, RequestChangeDir, RequestRun, RequestSetEnv, Response,
};
use crate::dsl::Chown;
use crate::sandbox::SandboxFile;
use crate::{RaptorError, RaptorResult};

#[derive(Debug)]
pub struct SandboxClient {
    proc: Child,
    conn: UnixStream,
}

impl SandboxClient {
    const START_TIMEOUT: Duration = Duration::from_secs(2);
    const CHECK_TIMEOUT: Duration = Duration::from_millis(100);

    #[must_use]
    pub const fn new(proc: Child, conn: UnixStream) -> Self {
        Self { proc, conn }
    }

    pub fn wait_for_startup(listen: UnixListener, proc: &mut Child) -> RaptorResult<UnixStream> {
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

    pub fn rpc(&mut self, req: &Request) -> RaptorResult<i32> {
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
        Ok(())
    }
}
