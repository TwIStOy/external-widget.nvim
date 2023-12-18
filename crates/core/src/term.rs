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
