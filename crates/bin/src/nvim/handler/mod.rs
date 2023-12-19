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

    async fn handle_notify(&self, name: String, _args: Vec<Value>, nvim: Nvim) {
        if name == "hover" {
            // process hover
        }
    }
}
