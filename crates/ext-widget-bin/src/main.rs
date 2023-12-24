use clap::{Args, Parser, Subcommand};
use ext_widget_core::{
    logger::install_logger,
    nvim::{start_parent, start_server},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start ex-widget in server mode.
    Serve(ServeOpts),
    /// Start ex-widget in embeded mode.
    Embed,
}

#[derive(Args)]
struct ServeOpts {
    /// The address to listen on.
    #[arg(long, default_value = "127.0.0.1:7000")]
    addr: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    install_logger()?;

    match cli.command {
        Commands::Serve(opts) => {
            let addr: String = opts.addr.parse()?;
            start_server(&addr).await?;
        }
        Commands::Embed => {
            start_parent().await?;
        }
    }

    Ok(())
}
