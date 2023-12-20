use std::time::Duration;

use anyhow::bail;
use async_trait::async_trait;
use external_widget_core::{
    kitty::{display_image, transmit_image, ID},
    nvim::{Nvim, NvimWriter},
    TermWriter,
};
use nvim_rs::{Handler, Neovim};
use rmpv::Value;
use tokio::time::sleep;
use tracing::info;

use self::hover::HoverHandler;

mod hover;

#[derive(Clone)]
pub struct NeovimHandler {}

#[async_trait]
impl Handler for NeovimHandler {
    type Writer = NvimWriter;

    async fn handle_request(
        &self, name: String, _args: Vec<Value>, _neovim: Neovim<Self::Writer>,
    ) -> Result<Value, Value> {
        info!("handle_request: {}", name);
        let res = match name.as_ref() {
            "start_hover" => self.process_req_start_hover(_args, _neovim).await,
            _ => unimplemented!(),
        };
        match res {
            Ok(v) => Ok(v),
            Err(e) => Err(Value::from(e.to_string())),
        }
    }

    async fn handle_notify(
        &self, name: String, args: Vec<Value>, nvim: Neovim<Self::Writer>,
    ) {
        println!("handle notify: {}, args: {:?}", name, args);
        let r = self.handle_notify_impl(name, args, nvim).await;
        if let Err(err) = r {
            println!("r: {}", err);
        }
    }
}

impl NeovimHandler {
    async fn handle_notify_impl(
        &self, name: String, args: Vec<Value>, nvim: Nvim,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

#[async_trait]
impl HoverHandler for NeovimHandler {}
