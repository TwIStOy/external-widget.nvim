mod handler;

use external_widget_core::nvim::{NvimSession, NvimWriter};
pub use handler::NeovimHandler;
use nvim_rs::Neovim;
use tokio::{
    io::{split, stdin, stdout},
    net::{TcpListener, TcpStream},
};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tracing::{error, info};

/// Start a TCP server on the given address.
pub(crate) async fn start_server(addr: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;

    tokio::spawn(async move {
        loop {
            let (tcp, addr) = listener.accept().await.unwrap();
            info!("Accepted connection, {:?}, {:?}", tcp, addr);
            create_neovim_from_tcp(tcp).await;
        }
    });

    Ok(())
}

pub(crate) async fn start_parent() -> anyhow::Result<()> {
    let handler = NeovimHandler {};
    let (_neovim, io) = Neovim::<NvimWriter>::new(
        stdin().compat(),
        Box::new(stdout().compat_write()),
        handler,
    );

    tokio::spawn(async move {
        if let Err(error) = io.await {
            if !error.is_channel_closed() {
                error!("Error: '{}'", error);
            }
        };
    });

    Ok(())
}

async fn create_neovim_from_tcp(tcp: TcpStream) {
    let handler = NeovimHandler {};
    let (reader, writer) = split(tcp);
    let (neovim, io) = Neovim::<NvimWriter>::new(
        reader.compat(),
        Box::new(writer.compat_write()),
        handler,
    );
    let session = NvimSession::new(neovim);

    tokio::spawn(async move {
        // TODO(hawtian): process neovim instance?
        if let Err(error) = io.await {
            if !error.is_channel_closed() {
                error!("Error: '{}'", error);
            }
        };
    });
}
