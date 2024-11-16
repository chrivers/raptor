use std::os::unix::net::{UnixListener, UnixStream};
use std::process::Child;

use crate::client::{FramedRead, FramedWrite, Request, Response};
use crate::sandbox::{ConsoleMode, Settings, SpawnBuilder};
use crate::{RaptorError, RaptorResult};

pub struct Sandbox {
    proc: Child,
    conn: UnixStream,
}

const SOCKET_PATH: &str = "/tmp/raptor";

impl Sandbox {
    pub fn new(layers: &[&str]) -> RaptorResult<Self> {
        if std::fs::exists(SOCKET_PATH)? {
            std::fs::remove_file(SOCKET_PATH)?;
        }

        let listen = UnixListener::bind(SOCKET_PATH)?;

        let proc = SpawnBuilder::new()
            .quiet()
            .with_sudo()
            .settings(Settings::False)
            .root_overlays(layers)
            .bind_ro("target/debug/nspawn-client", "/nspawn-client")
            .bind_ro(SOCKET_PATH, SOCKET_PATH)
            .console(ConsoleMode::ReadOnly)
            .directory(layers[0])
            .arg("/nspawn-client")
            .arg(SOCKET_PATH)
            .command()
            .spawn()?;

        let conn = listen.accept()?.0;

        Ok(Self { proc, conn })
    }

    pub fn run(&mut self, cmd: &[String]) -> RaptorResult<i32> {
        let req = Request::Run {
            arg0: cmd[0].clone(),
            argv: cmd.to_vec(),
        };
        self.conn.write_framed(&req)?;
        match self.conn.read_framed::<Response>()? {
            Response::Err(err) => Err(RaptorError::RunError(err)),
            Response::Ok(res) => Ok(res),
        }
    }

    pub fn close(&mut self) -> RaptorResult<()> {
        self.conn.write_framed(Request::Shutdown {})?;
        self.conn.shutdown(std::net::Shutdown::Write)?;
        self.proc.wait()?;
        Ok(())
    }
}
