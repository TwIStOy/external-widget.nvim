use std::fmt::Debug;

use futures::AsyncWrite;
use nvim_rs::Neovim;
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};
use tracing::info;

use crate::{
    env::in_tmux,
    nvim::NeovimSession,
    tmux::{tmux_escape_write, tmux_pane_tty},
};

pub struct TermWriter {
    inner: BufWriter<File>,
    tmux: bool,
    tty: String,
}

impl Debug for TermWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TermWriter")
            .field("tmux", &self.tmux)
            .field("tty", &self.tty)
            .finish()
    }
}

impl TermWriter {
    pub async fn new<W>(nvim: &Neovim<W>) -> anyhow::Result<Self>
    where
        W: AsyncWrite + Send + Unpin + 'static,
    {
        let tmux = in_tmux();
        let tty = if tmux {
            tmux_pane_tty().await?
        } else {
            NeovimSession::get_tty(nvim).await?
        };
        info!("tty: {}", tty);
        let writer =
            tokio::fs::OpenOptions::new().write(true).open(&tty).await?;
        let writer = BufWriter::new(writer);

        Ok(Self {
            inner: writer,
            tmux,
            tty,
        })
    }

    pub async fn new_tmux_tty(tty: &str, tmux: bool) -> anyhow::Result<Self> {
        let writer =
            tokio::fs::OpenOptions::new().write(true).open(tty).await?;
        let writer = BufWriter::new(writer);

        Ok(Self {
            inner: writer,
            tmux,
            tty: tty.to_string(),
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

    pub async fn flush(&mut self) -> anyhow::Result<()> {
        self.inner.flush().await?;
        Ok(())
    }
}
