// use std::sync::Arc;

// use anyhow::{bail, Context};
// use async_trait::async_trait;
// use nvim_rs::Neovim;
// use rmpv::Value;
// use tracing::instrument;

// use crate::nvim::{handler::NeovimService, NeovimSession, NvimWriter};

// struct StartHoverReq;

// #[instrument(skip(nvim))]
// async fn process_req_start_hover(
//     args: Vec<Value>, nvim: Neovim<NvimWriter>,
// ) -> anyhow::Result<Value> {
//     if args.len() != 2 {
//         bail!("hover expects 2 arguments, got {}", args.len());
//     }
//     let md = args[0]
//         .as_str()
//         .context("First args expect str")?
//         .to_string();
//     let lang = args[1]
//         .as_str()
//         .context("Second args expect str")?
//         .to_string();
//     if md.is_empty() {
//         bail!("hover expects non-empty markdown");
//     }
//     let id = ImageManager::alloc_id();
//     tokio::spawn(async move {
//         let image = build_hover_doc_image(&nvim, md, &lang).await;
//         match image {
//             Ok(image) => {
//                 let image =
//                     IMAGE_MANAGER.lock().new_image_from_id_buffer(id, image);
//                 image.render_at(5, 5).await.unwrap();
//             }
//             Err(err) => {
//                 warn!("Error building hover doc image: {}", err);
//                 nvim.err_writeln(&format!(
//                     "Error building hover doc image: {}",
//                     err
//                 ))
//                 .await
//                 .unwrap_or_else(|e| {
//                     warn!("Error writing to nvim: {}", e);
//                 });
//             }
//         }
//     });
//     Ok(Value::from(u32::from(id)))
// }

// #[async_trait]
// impl NeovimService for StartHoverReq {
//     // #[instrument(skip(self, nvim))]
//     async fn call(
//         &self, name: String, args: Vec<rmpv::Value>,
//         neovim: Neovim<NvimWriter>, session: Arc<NeovimSession>,
//     ) -> Result<Value, Value> {
//         match process_req_start_hover(args, neovim).await {
//             Ok(v) => Ok(v),
//             Err(e) => Err(Value::from(e.to_string())),
//         }
//     }
// }
