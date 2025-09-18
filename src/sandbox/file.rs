use std::io::{Error, ErrorKind, Write};

use camino::Utf8Path;

use crate::sandbox::FalconClient;
use crate::{RaptorError, RaptorResult};
use falcon::client::{Account, Request, RequestCloseFd, RequestCreateFile, RequestWriteFd};
use raptor_parser::ast::Chown;

#[derive(Debug)]
pub struct SandboxFile<'sb> {
    sandbox_client: &'sb mut FalconClient,
    fd: i32,
}

impl<'sb> SandboxFile<'sb> {
    pub fn new(
        sandbox_client: &'sb mut FalconClient,
        path: &Utf8Path,
        owner: Option<Chown>,
        mode: Option<u32>,
    ) -> RaptorResult<Self> {
        let Chown { user, group } = owner.unwrap_or_default();
        let fd = sandbox_client.rpc(&Request::CreateFile(RequestCreateFile {
            path: path.to_owned(),
            user: user.map(Account::Name),
            group: group.map(Account::Name),
            mode,
        }))?;
        Ok(Self { sandbox_client, fd })
    }
}

impl Write for SandboxFile<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.sandbox_client.rpc(&Request::WriteFd(RequestWriteFd {
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
            .sandbox_client
            .rpc(&Request::CloseFd(RequestCloseFd { fd: self.fd }));
    }
}
