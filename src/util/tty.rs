use std::os::fd::AsRawFd;

use nix::errno::Errno;
use nix::libc::{TIOCSWINSZ, ioctl, winsize};

pub trait TtyIoctl: AsRawFd {
    fn tty_set_window_size(&self, ws_size: winsize) -> Result<(), std::io::Error>;

    fn tty_set_size(&self, rows: u16, cols: u16) -> Result<(), std::io::Error> {
        self.tty_set_window_size(winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        })
    }
}

impl<T: AsRawFd> TtyIoctl for T {
    fn tty_set_window_size(&self, ws_size: winsize) -> Result<(), std::io::Error> {
        let res = unsafe { ioctl(self.as_raw_fd(), TIOCSWINSZ, &ws_size) };

        Ok(Errno::result(res).map(drop)?)
    }
}
