use std::{num::NonZeroU32, time::Duration};

use anyhow::bail;
use async_trait::async_trait;
use external_widget_core::{
    kitty::{delete_image, display_image, transmit_image, ID},
    nvim::{Nvim, NvimWriter},
    TermWriter,
};
use nvim_rs::Handler;
use rmpv::Value;
use tokio::time::sleep;

mod hover;

#[derive(Clone)]
pub struct NeovimHandler {}

#[async_trait]
impl Handler for NeovimHandler {
    type Writer = NvimWriter;

    async fn handle_request(
        &self, name: String, _args: Vec<Value>, _neovim: Nvim,
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

    async fn handle_notify(&self, name: String, args: Vec<Value>, nvim: Nvim) {
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
        if name == "hover" {
            if args.len() != 2 {
                bail!("hover expects 2 arguments, got {}", args.len());
            }
            let md = args[0].as_str().unwrap_or_default();
            let lang = args[1].as_str().unwrap_or_default();
            println!("md: {}, lang: {}", md, lang);
            let image =
                hover::build_hover_doc_image(&nvim, md.to_string(), lang)
                    .await?;
            tokio::fs::write("/tmp/test.png", &image).await?;
            let mut writer = TermWriter::new().await?;
            let id = ID(10.try_into().unwrap());
            transmit_image(&image, &mut writer, id).await?;
            sleep(Duration::from_millis(1000)).await;
            display_image(&mut writer, id).await?;
        }
        Ok(())
    }
}
