mod handler;
mod handlers;
mod highlight;
mod session;

use futures::AsyncWrite;
pub use highlight::HighlightInfos;
use nvim_rs::Neovim;
pub use session::NeovimSession;
use tokio::{
    io::{split, stdin, stdout},
    net::{TcpListener, TcpStream},
};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tracing::{error, info, instrument};

pub(crate) use handler::NeovimHandler;

type NvimWriter = Box<dyn AsyncWrite + Send + Unpin + 'static>;

/// Start a TCP server on the given address.
#[instrument]
pub async fn start_server(addr: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (tcp, addr) = listener.accept().await.unwrap();
        info!("Accepted connection, {:?}, {:?}", tcp, addr);
        create_neovim_from_tcp(tcp).await;
    }
}

pub async fn start_parent() -> anyhow::Result<()> {
    let handler = NeovimHandler::new();
    let (_neovim, io) = Neovim::<NvimWriter>::new(
        stdin().compat(),
        Box::new(stdout().compat_write()),
        handler,
    );

    if let Err(error) = io.await {
        if !error.is_channel_closed() {
            error!("Error: '{}'", error);
        }
    };

    Ok(())
}

async fn create_neovim_from_tcp(tcp: TcpStream) {
    let handler = NeovimHandler::new();
    let (reader, writer) = split(tcp);
    let (_neovim, io) = Neovim::<NvimWriter>::new(
        reader.compat(),
        Box::new(writer.compat_write()),
        handler,
    );

    tokio::spawn(async move {
        if let Err(error) = io.await {
            if !error.is_channel_closed() {
                error!("Error: '{}'", error);
            }
        };
    });
}
