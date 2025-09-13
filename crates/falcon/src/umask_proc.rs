use std::os::unix::process::CommandExt;
use std::process::Command;

use nix::sys::stat::{umask, Mode};

/// Command extension to set umask for spawned child process
pub trait Umask {
    fn umask(&mut self, umask: Mode) -> &mut Self;
}

impl Umask for Command {
    fn umask(&mut self, mode: Mode) -> &mut Self {
        unsafe {
            self.pre_exec(move || {
                umask(mode);
                Ok(())
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use nix::sys::stat::Mode;

    use crate::umask_proc::Umask;
    use crate::error::FalconResult;

    fn test_umask(mask: u32) -> FalconResult<()> {
        Command::new("/bin/sh")
            .arg("-c")
            .umask(Mode::from_bits_truncate(mask))
            .arg(format!("[ $(umask) = {mask:03o} ]"))
            .status()?;

        Ok(())
    }

    #[test]
    fn set_umask_000() -> FalconResult<()> {
        test_umask(0o000)
    }

    #[test]
    fn set_umask_007() -> FalconResult<()> {
        test_umask(0o007)
    }

    #[test]
    fn set_umask_022() -> FalconResult<()> {
        test_umask(0o022)
    }

    #[test]
    fn set_umask_027() -> FalconResult<()> {
        test_umask(0o027)
    }

    #[test]
    fn set_umask_777() -> FalconResult<()> {
        test_umask(0o777)
    }
}
