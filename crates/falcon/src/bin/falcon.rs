use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Write};
use std::os::fd::{AsFd, AsRawFd};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::net::UnixStream;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::process::Command;
use std::str::FromStr;

use nix::errno::Errno;
use nix::sys::stat::{umask, Mode};
use nix::unistd::{chown, fchown, Gid, Group, Uid, User};

use log::{debug, error, trace, LevelFilter};

use falcon::client::{
    Account, FramedRead, FramedWrite, Request, RequestChangeDir, RequestCloseFd, RequestCreateDir,
    RequestCreateFile, RequestRun, RequestSetEnv, RequestWriteFd, Response,
};
use falcon::umask_proc::Umask;

use falcon::error::{FalconError, FalconResult};

fn request_run(req: &RequestRun) -> FalconResult<i32> {
    debug!("Exec {} {:?}", req.arg0, &req.argv);
    Ok(Command::new(&req.argv[0])
        .arg0(&req.argv[0])
        .args(&req.argv[1..])
        .umask(Mode::S_IWGRP | Mode::S_IWOTH)
        .status()
        .map(ExitStatusExt::into_raw)?)
}

fn request_changedir(req: &RequestChangeDir) -> FalconResult<i32> {
    debug!("Chdir {:?}", req.cd);
    std::env::set_current_dir(&req.cd)
        .map_err(|err| Errno::from_raw(err.raw_os_error().unwrap()))?;
    Ok(0)
}

#[allow(clippy::unnecessary_wraps)]
fn request_setenv(req: &RequestSetEnv) -> FalconResult<i32> {
    debug!("Setenv {:?}={:?}", &req.key, &req.value);
    std::env::set_var(&req.key, &req.value);
    Ok(0)
}

fn uid_from_account(acct: &Account) -> FalconResult<Uid> {
    match acct {
        Account::Id(uid) => Ok(Uid::from_raw(*uid)),
        Account::Name(name) => {
            let res = User::from_name(name)?;
            if let Some(user) = res {
                debug!("resolved unix user {name:?} to {user:?}");
                Ok(user.uid)
            } else {
                error!("could not resolve unix user {name:?}");
                Err(FalconError::IoError(Error::new(
                    ErrorKind::NotFound,
                    "User not found",
                )))
            }
        }
    }
}

fn gid_from_account(acct: &Account) -> FalconResult<Gid> {
    match acct {
        Account::Id(uid) => Ok(Gid::from_raw(*uid)),
        Account::Name(name) => {
            let res = Group::from_name(name)?;
            if let Some(group) = res {
                debug!("resolved unix group {name:?} to {group:?}");
                Ok(group.gid)
            } else {
                error!("could not resolve unix group {name:?}");
                Err(FalconError::IoError(Error::new(
                    ErrorKind::NotFound,
                    "Group not found",
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

    pub fn create_file(&mut self, req: &RequestCreateFile) -> FalconResult<i32> {
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
            fchown(file.as_fd(), uid, gid)?;
        }

        self.0.insert(fd, file);
        Ok(fd)
    }

    #[allow(clippy::unused_self)]
    pub fn create_dir(&self, req: &RequestCreateDir) -> FalconResult<i32> {
        let path = &req.path;

        if req.parents {
            std::fs::create_dir_all(path)?;
        } else {
            std::fs::create_dir(path)?;
        }

        let uid = req.user.as_ref().map(uid_from_account).transpose()?;
        let gid = req.group.as_ref().map(gid_from_account).transpose()?;
        if uid.is_some() | gid.is_some() {
            chown(path.as_os_str(), uid, gid)?;
        }

        Ok(0)
    }

    fn write_fd(&mut self, req: &RequestWriteFd) -> FalconResult<i32> {
        self.get(req.fd)?.write_all(&req.data)?;
        Ok(0)
    }

    fn close_fd(&mut self, req: &RequestCloseFd) -> FalconResult<i32> {
        if self.0.remove(&req.fd).is_some() {
            Ok(0)
        } else {
            Err(FalconError::Errno(Errno::EBADF))?
        }
    }
}

fn main() -> FalconResult<()> {
    if let Ok(log_level) = std::env::var("FALCON_LOG_LEVEL") {
        let mut builder = colog::basic_builder();
        if let Ok(level) = LevelFilter::from_str(&log_level) {
            builder.filter_level(level);
        }
        builder.init();
    } else {
        colog::init();
    }

    let Ok(socket_name) = std::env::var("FALCON_SOCKET") else {
        error!("Missing environment setting: FALCON_SOCKET");
        std::process::exit(1);
    };

    let mut stream = UnixStream::connect(socket_name)?;

    let mut files = FileMap::new();

    umask(Mode::empty());

    loop {
        let req: Request = match stream.read_framed() {
            Ok(req) => req,
            Err(FalconError::IoError(err)) if err.kind() == ErrorKind::UnexpectedEof => break,
            Err(err) => {
                error!("Failed to read request: {err}");
                break;
            }
        };

        trace!("read request: {req:?}");

        let res = match req {
            Request::Run(req) => request_run(&req),
            Request::CreateFile(req) => files.create_file(&req),
            Request::CreateDir(req) => files.create_dir(&req),
            Request::WriteFd(req) => files.write_fd(&req),
            Request::CloseFd(req) => files.close_fd(&req),
            Request::ChangeDir(req) => request_changedir(&req),
            Request::SetEnv(req) => request_setenv(&req),
            Request::Shutdown => {
                break;
            }
        };

        let resp: Response = res.map_err(|err| match err {
            FalconError::IoError(err) => {
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
