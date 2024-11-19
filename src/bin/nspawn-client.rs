use std::collections::HashMap;
use std::fs::File;
use std::io::{ErrorKind, Write};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::process::Command;

use camino::Utf8Path;
use log::{error, info, trace};

use raptor::client::{FramedRead, FramedWrite, Request, Response};
use raptor::{RaptorError, RaptorResult};

fn request_run(arg0: &str, argv: &[String]) -> RaptorResult<i32> {
    info!("Exec {} {:?}", arg0, &argv);
    Ok(Command::new(&argv[0])
        .arg0(&argv[0])
        .args(&argv[1..])
        .status()
        .map(|code| code.into_raw())?)
}

fn request_create_file(files: &mut HashMap<i32, File>, path: &Utf8Path) -> RaptorResult<i32> {
    let file = File::create(path)?;
    let fd = file.as_raw_fd();
    files.insert(fd, file);
    Ok(fd)
}

fn request_write_fd(files: &mut HashMap<i32, File>, fd: i32, data: &[u8]) -> RaptorResult<i32> {
    if let Some(mut file) = files.get(&fd) {
        file.write_all(data)?;
        Ok(0)
    } else {
        Err(std::io::Error::new(ErrorKind::InvalidInput, "invalid fd"))?
    }
}

fn request_close_fd(files: &mut HashMap<i32, File>, fd: i32) -> RaptorResult<i32> {
    if files.remove(&fd).is_some() {
        Ok(0)
    } else {
        Err(std::io::Error::new(ErrorKind::InvalidInput, "invalid fd"))?
    }
}

fn main() -> RaptorResult<()> {
    colog::init();
    let Ok(socket_name) = std::env::var("RAPTOR_NSPAWN_SOCKET") else {
        error!("Missing environment setting: RAPTOR_NSPAWN_SOCKET");
        std::process::exit(1);
    };

    let mut stream = UnixStream::connect(socket_name)?;

    let mut files = HashMap::new();

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
            Request::Run { arg0, argv } => request_run(&arg0, &argv),
            Request::CreateFile { path } => request_create_file(&mut files, &path),
            Request::WriteFd { fd, data } => request_write_fd(&mut files, fd, &data),
            Request::CloseFd { fd } => request_close_fd(&mut files, fd),
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
