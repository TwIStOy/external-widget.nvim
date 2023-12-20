use logger::install_logger;
use nvim::{start_parent, start_server};

mod config;
mod logger;
mod nvim;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    install_logger()?;

    start_server("127.0.0.1:7000").await?;
    // start_parent().await?;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
