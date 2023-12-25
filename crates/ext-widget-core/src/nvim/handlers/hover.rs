use std::{cell::RefCell, rc::Rc, sync::Arc};

use anyhow::{bail, Context};
use async_trait::async_trait;
use futures::AsyncWrite;
use nvim_rs::Neovim;
use rmpv::Value;
use tracing::{instrument, warn};

use crate::{
    nvim::{handler::NeovimService, NeovimSession, NvimWriter, CONFIG},
    painting::{BoxBorder, BoxDecoration, Color, Padding, Renderer},
    term::image::{ImageManager, IMAGE_MANAGER},
    widgets::{BoxOptions, Container, MarkdownDocumentBuilder, WidgetTree},
};

struct StartHoverReq;

#[instrument(skip(nvim))]
async fn build_hover_doc_image<W>(
    nvim: Neovim<W>, session: Arc<NeovimSession>, md: &str,
) -> anyhow::Result<Vec<u8>>
where
    W: AsyncWrite + Send + Unpin + 'static,
{
    let (md_widget_builder, width, height) = {
        let cfg = CONFIG.lock();
        let md_widget_builder = MarkdownDocumentBuilder {
            nvim: nvim.clone(),
            session: session.clone(),
            normal_font: cfg.hover.normal_font.clone(),
            normal_font_size: cfg.hover.normal_font_size,
            mono_font: cfg.hover.mono_font.clone(),
            mono_font_size: cfg.hover.mono_font_size,
        };
        (
            md_widget_builder,
            cfg.hover.window.max_width,
            cfg.hover.window.max_height,
        )
    };
    let background_color = session
        .get_highlight_info(&nvim, "Normal")
        .await
        .unwrap_or_default();
    let widget = md_widget_builder.build(md)?;
    let background_color = background_color
        .guibg
        .unwrap_or_else(|| background_color.bg.unwrap_or_default());
    let mut container = Container::new(
        BoxDecoration {
            color: background_color,
            border: BoxBorder {
                width: 1.0,
                color: Color::new(0),
                radius: 0.0.into(),
            },
        },
        BoxOptions {
            padding: Padding::all(10.0.into()),
            ..Default::default()
        },
    );
    container.child = Some(widget);

    let mut widget_tree = WidgetTree::new();
    widget_tree.new_root(Rc::new(container))?;
    widget_tree.compute_layout(width, height)?;

    let renderer = Rc::new(RefCell::new(Renderer::new()?));
    widget_tree.paint(renderer.clone())?;

    let renderer = renderer.as_ref();
    let data = renderer.borrow_mut().snapshot_png_raw()?;

    Ok(data)
}

#[instrument(skip(nvim))]
async fn process_req_start_hover(
    args: Vec<Value>, nvim: Neovim<NvimWriter>, session: Arc<NeovimSession>,
) -> anyhow::Result<u32> {
    if args.len() != 1 {
        bail!("hover expects 2 arguments, got {}", args.len());
    }
    let md = args[0]
        .as_str()
        .context("First args expect str")?
        .to_string();
    if md.is_empty() {
        bail!("hover expects non-empty markdown");
    }
    let id = ImageManager::alloc_id();
    tokio::spawn(async move {
        let image =
            build_hover_doc_image(nvim.clone(), session.clone(), &md).await;
        match image {
            Ok(image) => {
                let image =
                    IMAGE_MANAGER.lock().new_image_from_id_buffer(id, image);
                let (x, y) = {
                    let cfg = CONFIG.lock();
                    (cfg.hover.window.x_offset, cfg.hover.window.y_offset)
                };
                let writer = session.get_tty_writer(&nvim).await.unwrap();
                let mut writer = writer.lock().await;
                image
                    .render_at(&mut writer, x.ceil() as u32, y.ceil() as u32)
                    .await
                    .unwrap();
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
    Ok(u32::from(id))
}

#[async_trait]
impl NeovimService for StartHoverReq {
    // #[instrument(skip(self, nvim))]
    async fn call(
        &self, _name: String, args: Vec<Value>, neovim: Neovim<NvimWriter>,
        session: Arc<NeovimSession>,
    ) -> Result<Value, Value> {
        match process_req_start_hover(args, neovim, session).await {
            Ok(v) => Ok(Value::from(v)),
            Err(e) => Err(Value::from(e.to_string())),
        }
    }
}
