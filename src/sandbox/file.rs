use std::io::{Error, ErrorKind, Write};

use camino::Utf8Path;

use crate::client::{Account, Request, RequestCloseFd, RequestCreateFile, RequestWriteFd};
use crate::dsl::Chown;
use crate::sandbox::Sandbox;
use crate::{RaptorError, RaptorResult};

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
