use std::collections::HashMap;
use std::fs::File;
use std::io::{ErrorKind, Write};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::process::Command;

use log::{error, info, trace};

use raptor::client::{
    FramedRead, FramedWrite, Request, RequestCloseFd, RequestCreateFile, RequestRun,
    RequestWriteFd, Response,
};
use raptor::{RaptorError, RaptorResult};

fn request_run(req: &RequestRun) -> RaptorResult<i32> {
    info!("Exec {} {:?}", req.arg0, &req.argv);
    Ok(Command::new(&req.argv[0])
        .arg0(&req.argv[0])
        .args(&req.argv[1..])
        .status()
        .map(|code| code.into_raw())?)
}

struct FileMap(HashMap<i32, File>);

impl FileMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get(&mut self, fd: i32) -> Result<&mut File, std::io::Error> {
        if let Some(file) = self.0.get_mut(&fd) {
            Ok(file)
        } else {
            Err(std::io::Error::new(ErrorKind::InvalidInput, "invalid fd"))?
        }
    }

    pub fn create_file(&mut self, req: &RequestCreateFile) -> RaptorResult<i32> {
        let file = File::create(&req.path)?;
        let fd = file.as_raw_fd();
        self.0.insert(fd, file);
        Ok(fd)
    }

    fn write_fd(&mut self, req: &RequestWriteFd) -> RaptorResult<i32> {
        self.get(req.fd)?.write_all(&req.data)?;
        Ok(0)
    }

    fn close_fd(&mut self, req: &RequestCloseFd) -> RaptorResult<i32> {
        if self.0.remove(&req.fd).is_some() {
            Ok(0)
        } else {
            Err(std::io::Error::new(ErrorKind::InvalidInput, "invalid fd"))?
        }
    }
}

fn main() -> RaptorResult<()> {
    colog::init();
    let Ok(socket_name) = std::env::var("RAPTOR_NSPAWN_SOCKET") else {
        error!("Missing environment setting: RAPTOR_NSPAWN_SOCKET");
        std::process::exit(1);
    };

    let mut stream = UnixStream::connect(socket_name)?;

    let mut files = FileMap::new();

    loop {
        let req: Request = match stream.read_framed() {
            Ok(req) => req,
            Err(RaptorError::IoError(err)) if err.kind() == ErrorKind::UnexpectedEof => break,
            Err(err) => {
                error!("Failed to read request: {err}");
                break;
            }
        };

        trace!("read request: {req:?}");

        let res = match req {
            Request::Run(req) => request_run(&req),
            Request::CreateFile(req) => files.create_file(&req),
            Request::WriteFd(req) => files.write_fd(&req),
            Request::CloseFd(req) => files.close_fd(&req),
            Request::Shutdown {} => {
                break;
            }
        };

        let resp = match res {
            Ok(code) => Response::Ok(code),
            Err(err) => {
                error!("Error: {err}");
                Response::Err(err.to_string())
            }
        };
        trace!("writing response: {resp:?}");
        stream.write_framed(resp)?;
    }

    Ok(())
}
