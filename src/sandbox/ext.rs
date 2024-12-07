use std::io::Write;

use camino::Utf8Path;

use crate::dsl::Chown;
use crate::sandbox::FalconClient;
use crate::RaptorResult;

pub trait SandboxExt {
    fn shell(&mut self, cmd: &[&str]) -> RaptorResult<()>;
    fn write_file(
        &mut self,
        path: impl AsRef<Utf8Path>,
        owner: Option<Chown>,
        mode: Option<u32>,
        data: impl AsRef<[u8]>,
    ) -> RaptorResult<()>;
}

impl SandboxExt for FalconClient {
    fn shell(&mut self, cmd: &[&str]) -> RaptorResult<()> {
        let mut args = vec!["/bin/sh", "-c"];
        args.extend(cmd);
        self.run(
            &args
                .into_iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
        )
    }

    fn write_file(
        &mut self,
        path: impl AsRef<Utf8Path>,
        owner: Option<Chown>,
        mode: Option<u32>,
        data: impl AsRef<[u8]>,
    ) -> RaptorResult<()> {
        let mut fd = self.create_file(path.as_ref(), owner, mode)?;
        fd.write_all(data.as_ref())?;
        drop(fd);
        Ok(())
    }
}
