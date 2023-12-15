//! A basic example. Mainly for use in a test, but also shows off some basic
//! functionality.
use std::{env, error::Error, fs};

use async_trait::async_trait;

use external_widget_core::nvim::hl_props_from_group;
use rmpv::Value;

use tokio::{io::Stdout, net::TcpListener};

use nvim_rs::error::LoopError;
use nvim_rs::{create::tokio as create, rpc::IntoVal, Handler, Neovim};
use tokio::io::{split, WriteHalf};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio_util::compat::{
    Compat, TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt,
};

#[derive(Clone)]
struct NeovimHandler {}

#[async_trait]
impl Handler for NeovimHandler {
    type Writer = Compat<WriteHalf<TcpStream>>;

    async fn handle_request(
        &self, name: String, _args: Vec<Value>,
        _neovim: Neovim<Compat<WriteHalf<TcpStream>>>,
    ) -> Result<Value, Value> {
        println!("handle_request: {}", name);
        match name.as_ref() {
            "ping" => {
                println!("ping");
                Ok(Value::from("pong"))
            }
            _ => unimplemented!(),
        }
    }

    async fn handle_notify(
        &self, name: String, _args: Vec<Value>,
        nvim: Neovim<<Self as Handler>::Writer>,
    ) {
        println!("handle_notify, {}", name);
        println!(
            "hl: {:?}",
            hl_props_from_group("Normal", &nvim).await.unwrap()
        );
    }
}

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
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:7000").await.unwrap();

    loop {
        let (tcp, addr) = listener.accept().await.unwrap();
        println!("Accepted connection, {:?}, {:?}", tcp, addr);
        tokio::spawn(async move {
            process_connection(tcp).await;
        });
    }
}
