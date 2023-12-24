use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use nvim_rs::{Handler, Neovim};
use rmpv::Value;
use tracing::{instrument, warn};

use super::{NeovimSession, NvimWriter};

#[derive(Clone)]
pub struct NeovimHandler {
    session: Arc<NeovimSession>,
    req_handlers: Arc<HashMap<String, Box<dyn NeovimService>>>,
    noti_handlers: Arc<HashMap<String, Box<dyn NeovimService>>>,
}

impl NeovimHandler {
    pub fn new() -> Self {
        Self {
            session: Arc::new(NeovimSession::new()),
            req_handlers: Arc::new(HashMap::new()),
            noti_handlers: Arc::new(HashMap::new()),
        }
    }

    pub async fn post_instance(
        &self, nvim: &Neovim<NvimWriter>,
    ) -> anyhow::Result<()> {
        self.session.post_instance(nvim).await?;
        Ok(())
    }
}

impl Default for NeovimHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
pub(super) trait NeovimService: Send + Sync {
    async fn call(
        &self, name: String, args: Vec<Value>, neovim: Neovim<NvimWriter>,
        session: Arc<NeovimSession>,
    ) -> Result<Value, Value>;
}

#[async_trait]
impl Handler for NeovimHandler {
    type Writer = NvimWriter;

    #[instrument(skip(self, neovim))]
    async fn handle_request(
        &self, name: String, args: Vec<Value>, neovim: Neovim<Self::Writer>,
    ) -> Result<Value, Value> {
        let handler = self.req_handlers.get(&name);
        match handler {
            Some(handler) => {
                handler.call(name, args, neovim, self.session.clone()).await
            }
            None => Err(Value::from(format!("Unknown request: {}", name))),
        }
    }

    #[instrument(skip(self, neovim))]
    async fn handle_notify(
        &self, name: String, args: Vec<Value>, neovim: Neovim<Self::Writer>,
    ) {
        let handler = self.noti_handlers.get(&name);
        match handler {
            Some(handler) => {
                if let Err(err) =
                    handler.call(name, args, neovim, self.session.clone()).await
                {
                    warn!("Error: {}", err);
                }
            }
            None => warn!("Unknown notify: {}", name),
        }
    }
}
