use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use nvim_rs::compat::tokio::Compat;
use nvim_rs::Handler;
use tokio::process::{Child, ChildStdin};

use crate::nvim::NeovimSession;

#[derive(Clone)]
struct NeovimHandler {
    pub session: Arc<NeovimSession>,
}

#[async_trait]
impl Handler for NeovimHandler {
    type Writer = Compat<ChildStdin>;
}

pub struct EmbedNvim {
    pub neovim: nvim_rs::Neovim<Compat<ChildStdin>>,
    pub child: Child,
}

impl EmbedNvim {
    pub async fn new() -> anyhow::Result<Self> {
        let (neovim, child) = start_embed_nvim().await?;
        Ok(Self { neovim, child })
    }
}

impl Drop for EmbedNvim {
    fn drop(&mut self) {
        let r = self.child.start_kill();
        if let Err(e) = r {
            eprintln!("Error killing child process: {}", e);
            return;
        }
        while self.child.try_wait().unwrap().is_none() {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}

pub async fn start_embed_nvim(
) -> anyhow::Result<(nvim_rs::Neovim<Compat<ChildStdin>>, Child)> {
    let handler = NeovimHandler {
        session: Arc::new(NeovimSession::new()),
    };
    let (neovim, io_handler, c) =
        nvim_rs::create::tokio::new_child(handler).await?;

    let nvim = neovim.clone();
    tokio::spawn(async move {
        match io_handler.await {
            Err(joinerr) => eprintln!("Error joining IO loop: '{}'", joinerr),
            Ok(Err(err)) => {
                if !err.is_reader_error() {
                    // One last try, since there wasn't an error with writing to the
                    // stream
                    nvim.err_writeln(&format!("Error: '{}'", err))
                        .await
                        .unwrap_or_else(|e| {
                            // We could inspect this error to see what was happening, and
                            // maybe retry, but at this point it's probably best
                            // to assume the worst and print a friendly and
                            // supportive message to our users
                            eprintln!("Well, dang... '{}'", e);
                        });
                }

                if !err.is_channel_closed() {
                    // Closed channel usually means neovim quit itself, or this plugin was
                    // told to quit by closing the channel, so it's not always an error
                    // condition.
                    eprintln!("Error: '{}'", err);

                    let mut source = err.source();

                    while let Some(e) = source {
                        eprintln!("Caused by: '{}'", e);
                        source = e.source();
                    }
                }
            }
            Ok(Ok(())) => {}
        }
    });

    Ok((neovim, c))
}
