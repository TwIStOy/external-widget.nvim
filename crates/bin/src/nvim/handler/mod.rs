use anyhow::bail;
use async_trait::async_trait;
use external_widget_core::nvim::{Nvim, NvimWriter};
use nvim_rs::Handler;
use rmpv::Value;

mod hover;

#[derive(Clone)]
struct NeovimHandler {}

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
        let _ = self.handle_notify_impl(name, args, nvim).await;
    }
}

impl NeovimHandler {
    async fn handle_notify_impl(
        &self, name: String, args: Vec<Value>, nvim: Nvim,
    ) -> anyhow::Result<()> {
        if name == "hover" {
            if args.len() != 2 {
                bail!("hover expects 2 arguments");
            }
            let md = args[0].as_str().unwrap_or_default();
            let lang = args[1].as_str().unwrap_or_default();
            let image =
                hover::build_hover_doc_image(&nvim, md.to_string(), lang)
                    .await
                    .unwrap_or_default();
        }
        Ok(())
    }
}
