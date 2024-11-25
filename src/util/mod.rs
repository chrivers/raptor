use std::io::{BufWriter, Read, Write};

use crate::RaptorResult;

const BUFFER_SIZE: usize = 128 * 1024;

pub fn io_fast_copy(mut src: impl Read, dst: impl Write) -> RaptorResult<()> {
    let mut dst = BufWriter::with_capacity(BUFFER_SIZE, dst);
    std::io::copy(&mut src, &mut dst)?;
    Ok(())
}

pub mod capture_proc_fd;
pub mod kwargs;
pub mod umask_proc;
