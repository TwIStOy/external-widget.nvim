use std::sync::Arc;

use async_trait::async_trait;
use nvim_rs::Neovim;
use rmpv::{ext::from_value, Value};
use tracing::instrument;

use crate::nvim::{
    handler::NeovimService, ExtWidgetConfig, NeovimSession, NvimWriter, CONFIG,
};

#[derive(Debug)]
pub(crate) struct ConfigNotify;

#[async_trait]
impl NeovimService for ConfigNotify {
    #[instrument(skip(self, _neovim, _session))]
    async fn call(
        &self, _name: String, args: Vec<Value>, _neovim: Neovim<NvimWriter>,
        _session: Arc<NeovimSession>,
    ) -> Result<Value, Value> {
        if args.len() != 1 {
            return Err(Value::from(format!(
                "config_notify expects 1 argument, got {}",
                args.len()
            )));
        }
        let new_config: ExtWidgetConfig =
            from_value(args[0].clone()).map_err(|e| {
                Value::from(format!("Deserialize config failed: {}", e))
            })?;
        *CONFIG.lock() = new_config;
        Ok(Value::from(true))
    }
}
