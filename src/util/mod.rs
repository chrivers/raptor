use std::io::{BufWriter, Read, Write};

use camino::{Utf8Path, Utf8PathBuf};

use crate::RaptorResult;

const BUFFER_SIZE: usize = 128 * 1024;

pub fn io_fast_copy(mut src: impl Read, dst: impl Write) -> RaptorResult<()> {
    let mut dst = BufWriter::with_capacity(BUFFER_SIZE, dst);
    std::io::copy(&mut src, &mut dst)?;
    Ok(())
}

pub trait SafeParent {
    fn try_parent(&self) -> RaptorResult<&Utf8Path>;
}

impl SafeParent for Utf8Path {
    fn try_parent(&self) -> RaptorResult<&Utf8Path> {
        self.parent()
            .ok_or_else(|| crate::RaptorError::BadPathNoParent(self.into()))
    }
}

impl SafeParent for Utf8PathBuf {
    fn try_parent(&self) -> RaptorResult<&Utf8Path> {
        self.parent()
            .ok_or_else(|| crate::RaptorError::BadPathNoParent(self.into()))
    }
}

pub mod capture_proc_fd;
pub mod clapcolor;
pub mod kwargs;
pub mod umask_proc;
