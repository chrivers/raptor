use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::RaptorResult;

/// Adapted from rust stdlib (private func)
pub fn cvt(t: libc::c_int) -> std::io::Result<libc::c_int> {
    if t == -1 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(t)
    }
}

/// Construct an anonymous (read, write) pipe
///
/// Adapted from rust stdlib (private func)
pub fn pipe() -> RaptorResult<(OwnedFd, OwnedFd)> {
    let mut fds = [0; 2];
    unsafe {
        cvt(libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC))?;
        Ok((OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])))
    }
}

/// Command extension to hook a specific file descriptor before executing
/// process, using `libc::dup2()`
pub trait HookFd {
    fn hook_fd(&mut self, fd: i32, dst: (impl AsRawFd + Send + Sync + 'static)) -> &mut Self;
}

impl HookFd for Command {
    fn hook_fd(&mut self, fd: i32, dst: (impl AsRawFd + Send + Sync + 'static)) -> &mut Self {
        unsafe { self.pre_exec(move || cvt(libc::dup2(dst.as_raw_fd(), fd)).map(|_| ())) }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use std::process::{Command, Stdio};

    use crate::util::capture_proc_fd::{pipe, HookFd};
    use crate::RaptorResult;

    #[test]
    fn test_hook_stdout() -> RaptorResult<()> {
        let (read, write) = pipe()?;

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
        let (read, write) = pipe()?;

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
        let (read, write) = pipe()?;

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
