use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::net::UnixStream;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::process::Command;

use nix::errno::Errno;
use nix::sys::stat::{umask, Mode};
use nix::unistd::{fchown, Gid, Group, Uid, User};

use log::{error, info, trace};

use raptor::client::{
    Account, FramedRead, FramedWrite, Request, RequestChangeDir, RequestCloseFd, RequestCreateFile,
    RequestRun, RequestSetEnv, RequestWriteFd, Response,
};
use raptor::util::umask_proc::Umask;
use raptor::{RaptorError, RaptorResult};

fn request_run(req: &RequestRun) -> RaptorResult<i32> {
    info!("Exec {} {:?}", req.arg0, &req.argv);
    Ok(Command::new(&req.argv[0])
        .arg0(&req.argv[0])
        .args(&req.argv[1..])
        .umask(Mode::S_IWGRP | Mode::S_IWOTH)
        .status()
        .map(ExitStatusExt::into_raw)?)
}

fn request_changedir(req: &RequestChangeDir) -> RaptorResult<i32> {
    info!("Chdir {:?}", req.cd);
    std::env::set_current_dir(&req.cd)
        .map_err(|err| Errno::from_raw(err.raw_os_error().unwrap()))?;
    Ok(0)
}

fn request_setenv(req: &RequestSetEnv) {
    info!("Setenv {:?}={:?}", &req.key, &req.value);
    std::env::set_var(&req.key, &req.value);
}

fn uid_from_account(acct: &Account) -> RaptorResult<Uid> {
    match acct {
        Account::Id(uid) => Ok(Uid::from_raw(*uid)),
        Account::Name(name) => {
            let res = User::from_name(name)?;
            if let Some(user) = res {
                Ok(user.uid)
            } else {
                Err(RaptorError::IoError(Error::new(
                    ErrorKind::NotFound,
                    "User not found",
                )))
            }
        }
    }
}

fn gid_from_account(acct: &Account) -> RaptorResult<Gid> {
    match acct {
        Account::Id(uid) => Ok(Gid::from_raw(*uid)),
        Account::Name(name) => {
            let res = Group::from_name(name)?;
            if let Some(group) = res {
                Ok(group.gid)
            } else {
                Err(RaptorError::IoError(Error::new(
                    ErrorKind::NotFound,
                    "User not found",
                )))
            }
        }
    }
}

struct FileMap(HashMap<i32, File>);

impl FileMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get(&mut self, fd: i32) -> Result<&mut File, Error> {
        if let Some(file) = self.0.get_mut(&fd) {
            Ok(file)
        } else {
            Err(Error::new(ErrorKind::InvalidInput, "invalid fd"))?
        }
    }

    pub fn create_file(&mut self, req: &RequestCreateFile) -> RaptorResult<i32> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(req.mode.unwrap_or(0o750))
            .open(&req.path)?;
        let fd = file.as_raw_fd();

        let uid = req.user.as_ref().map(uid_from_account).transpose()?;
        let gid = req.group.as_ref().map(gid_from_account).transpose()?;
        if uid.is_some() | gid.is_some() {
            fchown(fd, uid, gid)?;
        }

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
            Err(RaptorError::SandboxRequestError(Errno::EBADF))?
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

    umask(Mode::empty());

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
            Request::Shutdown => {
                break;
            }
            Request::ChangeDir(req) => request_changedir(&req),
            Request::SetEnv(req) => {
                request_setenv(&req);
                Ok(0)
            }
        };

        let resp: Response = res.map_err(|err| match err {
            RaptorError::IoError(err) => {
                error!("Error: {err}");
                err.raw_os_error().unwrap_or(Errno::EIO as i32)
            }
            err => {
                error!("Error: {err}");
                Errno::EIO as i32
            }
        });

        trace!("writing response: {resp:?}");
        stream.write_framed(resp)?;
    }

    Ok(())
}
