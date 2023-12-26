use libc::winsize;
use rustix::{
    fd::{AsRawFd, BorrowedFd, RawFd},
    termios::{isatty, tcgetwinsize},
};

use crate::nvim::NvimTermSize;

#[derive(Debug, Clone)]
pub struct TermSizeInfo {
    pub screen_width: f32,
    pub screen_height: f32,
    pub cell_width: f32,
    pub cell_height: f32,
    pub rows: i32,
    pub cols: i32,
}

impl TermSizeInfo {
    fn new(win: winsize) -> Self {
        let screen_width = win.ws_xpixel as f32;
        let screen_height = win.ws_ypixel as f32;
        let rows = win.ws_row as i32;
        let cols = win.ws_col as i32;
        let cell_width = screen_width / cols as f32;
        let cell_height = screen_height / rows as f32;
        Self {
            screen_width,
            screen_height,
            cell_width,
            cell_height,
            rows,
            cols,
        }
    }

    pub fn new_from_nvim_term(win: NvimTermSize) -> Self {
        let screen_width = win.xpixel as f32;
        let screen_height = win.ypixel as f32;
        let rows = win.row as i32;
        let cols = win.col as i32;
        let cell_width = screen_width / cols as f32;
        let cell_height = screen_height / rows as f32;
        Self {
            screen_width,
            screen_height,
            cell_width,
            cell_height,
            rows,
            cols,
        }
    }
}

fn terminal_size_using_fd(fd: RawFd) -> Option<winsize> {
    let fd = unsafe { BorrowedFd::borrow_raw(fd) };
    if !isatty(fd) {
        return None;
    }
    let winsize = tcgetwinsize(fd).ok()?;
    Some(winsize)
}

pub fn get_term_size_info_fd(fd: RawFd) -> Option<TermSizeInfo> {
    if let Some(size) = terminal_size_using_fd(fd) {
        Some(TermSizeInfo::new(size))
    } else {
        None
    }
}

pub fn get_term_size_info() -> Option<TermSizeInfo> {
    if let Some(size) = terminal_size_using_fd(std::io::stdout().as_raw_fd()) {
        Some(TermSizeInfo::new(size))
    } else if let Some(size) =
        terminal_size_using_fd(std::io::stderr().as_raw_fd())
    {
        Some(TermSizeInfo::new(size))
    } else if let Some(size) =
        terminal_size_using_fd(std::io::stdin().as_raw_fd())
    {
        Some(TermSizeInfo::new(size))
    } else {
        None
    }
}
