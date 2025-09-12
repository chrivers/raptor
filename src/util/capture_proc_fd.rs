use std::os::fd::{AsFd, IntoRawFd, OwnedFd};
use std::os::unix::process::CommandExt;
use std::process::Command;

/// Command extension to hook a specific file descriptor before executing
/// process, using `nix::unistd::dup2()`
pub trait HookFd {
    fn hook_fd(&mut self, fd: i32, dst: OwnedFd) -> &mut Self;
}

impl HookFd for Command {
    fn hook_fd(&mut self, fd: i32, dst: OwnedFd) -> &mut Self {
        unsafe {
            self.pre_exec(move || {
                // Call .into_raw_fd() to unwrap the returned file handle, so we
                // don't immediately drop the dup'ed file handle.
                let _ = nix::unistd::dup2_raw(dst.as_fd(), fd)?.into_raw_fd();

                Ok(())
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use std::process::{Command, Stdio};

    use nix::fcntl::OFlag;

    use crate::util::capture_proc_fd::HookFd;
    use crate::RaptorResult;

    #[test]
    fn test_hook_stdout() -> RaptorResult<()> {
        let (read, write) = nix::unistd::pipe2(OFlag::O_CLOEXEC)?;

        let mut proc = Command::new("/bin/sh")
            .arg("-c")
            .arg("echo 1 > /dev/stdout; echo 2 > /dev/stderr")
            .hook_fd(1, write)
            .stderr(Stdio::piped())
            .spawn()?;

        let mut output = String::new();

        let mut stderr = proc.stderr.take().unwrap();
        let mut reader: File = read.into();

        reader.read_to_string(&mut output)?;
        assert_eq!(output, "1\n");

        output.clear();

        stderr.read_to_string(&mut output)?;
        assert_eq!(output, "2\n");

        proc.wait()?;

        Ok(())
    }

    #[test]
    fn test_hook_stderr() -> RaptorResult<()> {
        let (read, write) = nix::unistd::pipe2(OFlag::O_CLOEXEC)?;

        let mut proc = Command::new("/bin/sh")
            .arg("-c")
            .arg("echo 1 > /dev/stdout; echo 2 > /dev/stderr")
            .stdout(Stdio::piped())
            .hook_fd(2, write)
            .spawn()?;

        let mut output = String::new();
        let mut stdout = proc.stdout.take().unwrap();

        stdout.read_to_string(&mut output)?;
        assert_eq!(output, "1\n");

        output.clear();

        let mut reader: File = read.into();
        reader.read_to_string(&mut output)?;
        assert_eq!(output, "2\n");

        proc.wait()?;

        Ok(())
    }

    #[test]
    fn test_hook_fd_3() -> RaptorResult<()> {
        let (read, write) = nix::unistd::pipe2(OFlag::O_CLOEXEC)?;

        let mut proc = Command::new("/bin/sh")
            .arg("-c")
            .arg("echo 1 > /dev/stdout; echo 2 > /dev/stderr; echo 3 >&3")
            .hook_fd(3, write)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut output = String::new();

        let mut stdout = proc.stdout.take().unwrap();
        let mut stderr = proc.stderr.take().unwrap();
        let mut reader: File = read.into();

        stdout.read_to_string(&mut output)?;
        assert_eq!(output, "1\n");

        output.clear();

        stderr.read_to_string(&mut output)?;
        assert_eq!(output, "2\n");

        output.clear();

        reader.read_to_string(&mut output)?;
        assert_eq!(output, "3\n");

        proc.wait()?;

        Ok(())
    }
}
