mod hover_doc;

use anyhow::bail;
use async_trait::async_trait;
use external_widget_core::{
    kitty::transmit_image, nvim::Nvim, Image, ImageManager, TermWriter,
    IMAGE_MANAGER,
};
pub use hover_doc::build_hover_doc_image;
use rmpv::Value;
use tracing::warn;

#[async_trait]
pub(super) trait HoverHandler {
    /// Returns the image's id immediately.
    /// Expect name: "start_hover"
    async fn process_req_start_hover(
        &self, args: Vec<Value>, nvim: Nvim,
    ) -> anyhow::Result<Value> {
        if args.len() != 2 {
            bail!("hover expects 2 arguments, got {}", args.len());
        }
        let md = args[0].as_str().unwrap_or_default().to_string();
        let lang = args[1].as_str().unwrap_or_default().to_string();
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
                    image.transmit().await.unwrap();
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

    async fn process_hover_notify(
        &self, args: Vec<Value>, nvim: Nvim,
    ) -> anyhow::Result<()> {
        if args.len() != 2 {
            bail!("hover expects 2 arguments, got {}", args.len());
        }
        let md = args[0].as_str().unwrap_or_default();
        let lang = args[1].as_str().unwrap_or_default();
        let image = build_hover_doc_image(&nvim, md.to_string(), lang).await?;
        let image = IMAGE_MANAGER.lock().new_image_from_buffer(image);
        Ok(())
    }
}
