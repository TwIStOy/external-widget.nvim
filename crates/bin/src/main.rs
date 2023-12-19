use std::{env, error::Error, fs};

use async_trait::async_trait;
mod config;
mod nvim;

use external_widget_core::kitty::{display_image, transmit_image, ID};
use external_widget_core::nvim::hl_props_from_group;
use external_widget_core::pango::MarkupProperties;
use external_widget_core::{term_get_size, TermWriter, Widget};
use external_widget_widgets::{render_widget_tree_to_buf, MdDoc, MdDocOpts};
use nvim::NeovimHandler;
use rmpv::Value;

use tokio::time::sleep;
use tokio::{io::Stdout, net::TcpListener};

use nvim_rs::error::LoopError;
use nvim_rs::{create::tokio as create, rpc::IntoVal, Handler, Neovim};
use tokio::io::{split, WriteHalf};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio_util::compat::{
    Compat, TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt,
};

fn receive_tcp(
    mut tcp: TcpStream,
) -> std::io::Result<(
    Neovim<Compat<WriteHalf<TcpStream>>>,
    JoinHandle<Result<(), Box<LoopError>>>,
)> {
    let handler: NeovimHandler = NeovimHandler {};
    let (reader, writer) = split(tcp);
    let (neovim, io) = Neovim::<Compat<WriteHalf<TcpStream>>>::new(
        reader.compat(),
        writer.compat_write(),
        handler,
    );
    let io_handle = tokio::spawn(io);
    Ok((neovim, io_handle))
}

async fn process_connection(tcp: TcpStream) {
    let (nvim, io_handler) = receive_tcp(tcp).unwrap();
    println!("Created neovim instance");

    // Any error should probably be logged, as stderr is not visible to users.
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7000").await.unwrap();

    loop {
        let (tcp, addr) = listener.accept().await.unwrap();
        println!("Accepted connection, {:?}, {:?}", tcp, addr);
        tokio::spawn(async move {
            process_connection(tcp).await;
        });
    }
}

// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     let md = tokio::fs::read_to_string("/tmp/test.md").await?;
//     let opts = MdDocOpts {
//         md,
//         highlights: HashMap::new(),
//         normal_font: "MonoLisa".to_string(),
//         mono_font: "MonoLisa".to_string(),
//         font_size: "14pt".to_string(),
//     };
//     let widget = Rc::new(MdDoc::new(opts)?);
//     let image = render_widget_tree_to_buf(widget, 1000, 1000)?;
//     tokio::fs::write("/tmp/test.png", &image).await?;
//     let mut writer = TermWriter::new_tmux_tty("/dev/pts/9", true).await?;
//     let id = ID(10.try_into().unwrap());
//     transmit_image(&image, &mut writer, id).await?;
//     sleep(Duration::from_millis(100)).await;
//     display_image(&mut writer, id).await?;
//     Ok(())
// }
