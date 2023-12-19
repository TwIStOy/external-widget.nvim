use anyhow::Context;
use tokio::{
    fs::File,
    io::{AsyncWrite, AsyncWriteExt, BufWriter},
};

use crate::{get_tty, in_tmux, tmux_escape_write, tmux_pane_tty};

pub async fn write_to_tty(data: &str) -> anyhow::Result<()> {
    let tty = if in_tmux() {
        tmux_pane_tty().await?
    } else {
        get_tty().await?
    };

    let tty = tokio::fs::OpenOptions::new().write(true).open(tty).await?;
    let mut writer = tokio::io::BufWriter::new(tty);
    writer.write_all(data.as_bytes()).await?;
    writer.flush().await?;

    Ok(())
}

pub async fn term_sync_start() -> anyhow::Result<()> {
    write_to_tty("\x1b[?2026h").await
}

pub async fn term_sync_end() -> anyhow::Result<()> {
    write_to_tty("\x1b[?2026l").await
}

pub struct TermWriter {
    inner: BufWriter<File>,
    tmux: bool,
}

impl TermWriter {
    pub async fn new() -> anyhow::Result<Self> {
        let tmux = in_tmux();
        let tty = if tmux {
            tmux_pane_tty().await?
        } else {
            get_tty().await?
        };
        let writer =
            tokio::fs::OpenOptions::new().write(true).open(tty).await?;
        let writer = BufWriter::new(writer);

        Ok(Self {
            inner: writer,
            tmux,
        })
    }

    pub async fn write_all(
        &mut self, buf: &[u8], escape: bool,
    ) -> anyhow::Result<()> {
        if escape && self.tmux {
            tmux_escape_write(buf, &mut self.inner).await
        } else {
            self.inner.write_all(buf).await?;
            Ok(())
        }
    }
}

pub fn term_get_size() -> anyhow::Result<(usize, usize)> {
    term_size::dimensions().context("Cannot get terminal size")
}

// function updateSize() {
//     let ffi = require("ffi") as AnyMod;

//     ffi.cdef(`
//       typedef struct {
//         unsigned short row;
//         unsigned short col;
//         unsigned short xpixel;
//         unsigned short ypixel;
//       } winsize;
//       int ioctl(int, int, ...);
//     `);

//     let TIOCGWINSZ = null;
//     if (vim.fn.has("linux") == 1) {
//       TIOCGWINSZ = 0x5413;
//     } else if (vim.fn.has("mac") == 1) {
//       TIOCGWINSZ = 0x40087468;
//     } else if (vim.fn.has("bsd") == 1) {
//       TIOCGWINSZ = 0x40087468;
//     }
//     let sz: {
//       row: number;
//       col: number;
//       xpixel: number;
//       ypixel: number;
//     } = ffi.new("winsize");
//     assert(ffi.C.ioctl(1, TIOCGWINSZ, sz) == 0, "Failed to get terminal size");
//     _cached_size = {
//       screen_x: sz.xpixel,
//       screen_y: sz.ypixel,
//       screen_cols: sz.col,
//       screen_rows: sz.row,
//       cell_width: sz.xpixel / sz.col,
//       cell_height: sz.ypixel / sz.row,
//     };
//   }
