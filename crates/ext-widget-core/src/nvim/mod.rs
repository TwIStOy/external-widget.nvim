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
        if let Err(e) = create_neovim_from_tcp(tcp).await {
            error!("Error: '{}'", e);
        }
    }
}

pub async fn start_parent() -> anyhow::Result<()> {
    let handler = NeovimHandler::new();
    let (neovim, io) = Neovim::<NvimWriter>::new(
        stdin().compat(),
        Box::new(stdout().compat_write()),
        handler.clone(),
    );
    handler.post_instance(&neovim).await?;

    if let Err(error) = io.await {
        if !error.is_channel_closed() {
            error!("Error: '{}'", error);
        }
    };

    Ok(())
}

async fn create_neovim_from_tcp(tcp: TcpStream) -> anyhow::Result<()> {
    let handler = NeovimHandler::new();
    let (reader, writer) = split(tcp);
    let (neovim, io) = Neovim::<NvimWriter>::new(
        reader.compat(),
        Box::new(writer.compat_write()),
        handler.clone(),
    );
    handler.post_instance(&neovim).await?;

    tokio::spawn(async move {
        if let Err(error) = io.await {
            if !error.is_channel_closed() {
                error!("Error: '{}'", error);
            }
        };
    });

    Ok(())
}
