mod hover_doc;

use std::num::NonZeroU32;

use anyhow::{bail, Context};
use async_trait::async_trait;
use external_widget_core::{
    kitty::ID, nvim::Nvim, ImageManager, IMAGE_MANAGER,
};
pub use hover_doc::build_hover_doc_image;
use rmpv::Value;
use tracing::{instrument, warn};

#[async_trait]
pub(super) trait HoverHandler {
    /// Returns the image's id immediately.
    /// Expect name: "start_hover"
    #[instrument(skip(self, nvim))]
    async fn process_req_start_hover(
        &self, args: Vec<Value>, nvim: Nvim,
    ) -> anyhow::Result<Value> {
        if args.len() != 2 {
            bail!("hover expects 2 arguments, got {}", args.len());
        }
        let md = args[0]
            .as_str()
            .context("First args expect str")?
            .to_string();
        let lang = args[1]
            .as_str()
            .context("Second args expect str")?
            .to_string();
        if md.is_empty() {
            bail!("hover expects non-empty markdown");
        }
        let id = ImageManager::alloc_id();
        tokio::spawn(async move {
            let image = build_hover_doc_image(&nvim, md, &lang).await;
            match image {
                Ok(image) => {
                    let image = IMAGE_MANAGER
                        .lock()
                        .new_image_from_id_buffer(id, image);
                    image.render_at(5, 5).await.unwrap();
                }
                Err(err) => {
                    warn!("Error building hover doc image: {}", err);
                    nvim.err_writeln(&format!(
                        "Error building hover doc image: {}",
                        err
                    ))
                    .await
                    .unwrap_or_else(|e| {
                        warn!("Error writing to nvim: {}", e);
                    });
                }
            }
        });
        Ok(Value::from(u32::from(id)))
    }

    /// Expect name: "stop_hover"
    #[instrument(skip(self, nvim))]
    async fn process_req_stop_hover(
        &self, args: Vec<Value>, nvim: Nvim,
    ) -> anyhow::Result<Value> {
        if args.len() != 1 {
            bail!("stop_hover expects 1 argument, got {}", args.len());
        }
        let id: ID = ID(NonZeroU32::try_from(
            args[0].as_u64().context("Expect u64")? as u32,
        )?);
        tokio::spawn(async move {
            let image = IMAGE_MANAGER.lock().find_image(id.0);
            if let Some(image) = image {
                if let Err(e) = image.delete_image().await {
                    nvim.err_writeln(&format!("Error deleting image: {}", e))
                        .await
                        .unwrap_or_else(|e| {
                            warn!("Error writing to nvim: {}", e);
                        });
                }
            }
        });
        Ok(Value::from(u32::from(id.0)))
    }
}
