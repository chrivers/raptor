use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use camino::{Utf8Path, Utf8PathBuf};

use crate::RaptorResult;

const BUFFER_SIZE: usize = 128 * 1024;

pub fn io_fast_copy(mut src: impl Read, dst: impl Write) -> RaptorResult<()> {
    let mut dst = BufWriter::with_capacity(BUFFER_SIZE, dst);
    std::io::copy(&mut src, &mut dst)?;
    Ok(())
}

pub fn copy_file(from: impl AsRef<Utf8Path>, to: impl AsRef<Utf8Path>) -> RaptorResult<()> {
    let src = File::open(from.as_ref())?;
    let mode = src.metadata()?.permissions().mode();
    let dst = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(mode)
        .open(to.as_ref())?;

    io_fast_copy(src, dst)
}

pub fn link_or_copy_file(from: impl AsRef<Utf8Path>, to: impl AsRef<Utf8Path>) -> RaptorResult<()> {
    std::fs::hard_link(from.as_ref(), to.as_ref()).or_else(|_| copy_file(from, to))
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
pub mod module_name;
