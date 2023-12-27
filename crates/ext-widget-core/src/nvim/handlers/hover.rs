use std::{cell::RefCell, num::NonZeroU32, rc::Rc, sync::Arc};

use anyhow::{bail, Context};
use async_trait::async_trait;
use futures::AsyncWrite;
use nvim_rs::Neovim;
use rmpv::Value;
use tracing::{info, instrument, warn};

use crate::{
    nvim::{handler::NeovimService, NeovimSession, NvimWriter, CONFIG},
    painting::{BoxBorder, BoxDecoration, Color, Padding, RectSize, Renderer},
    term::{
        image::{ImageManager, IMAGE_MANAGER},
        TermSizeInfo,
    },
    widgets::{BoxOptions, Container, MarkdownDocumentBuilder, WidgetTree},
};

async fn build_hover_doc_image<W>(
    nvim: Neovim<W>, session: Arc<NeovimSession>, md: &str,
) -> anyhow::Result<(Vec<u8>, RectSize<f32>)>
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
    let widget = md_widget_builder.build(md).await?;
    let background_color = background_color
        .guibg
        .unwrap_or_else(|| background_color.bg.unwrap_or_default());
    let container = Container::new_with_child(
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
        widget,
    );

    let mut widget_tree = WidgetTree::new();
    widget_tree.new_root(Rc::new(container))?;
    widget_tree.compute_layout(width, height)?;
    let image_size = widget_tree.result_size()?;

    let renderer =
        Rc::new(RefCell::new(Renderer::new(width as u32, height as u32)?));
    widget_tree.paint(renderer.clone())?;

    for line in widget_tree.debug_tree()? {
        info!("{}", line);
    }

    let renderer = renderer.as_ref();
    let data = renderer.borrow_mut().snapshot_png_raw()?;

    info!("Data len: {}", data.len());

    Ok((data, image_size))
}

#[instrument(skip(nvim))]
async fn image_offset_to_term(
    nvim: &Neovim<NvimWriter>, image_size: RectSize<f32>, offset: (i32, i32),
) -> anyhow::Result<(u32, u32)> {
    let term_size = NeovimSession::get_term_size(nvim).await?;
    let term_size = TermSizeInfo::new_from_nvim_term(term_size);
    let image_cell_width =
        (image_size.width / term_size.cell_width).ceil() as i32;
    let image_cell_height =
        (image_size.height / term_size.cell_height).ceil() as i32;
    let (cur_row, cur_col) =
        NeovimSession::cursor_position_to_client(nvim).await?;
    let x_offset = if cur_col + offset.0 + image_cell_width > term_size.cols {
        term_size.cols - image_cell_width
    } else {
        cur_col + offset.0
    };
    let y_offset = if cur_row + offset.1 + image_cell_height > term_size.rows {
        term_size.rows - image_cell_height
    } else {
        cur_row + offset.1
    };
    // let x_offset =
    //     (term_size.cols - cur_col - image_cell_width + offset.0).min(0);
    // let y_offset =
    //     (term_size.rows - cur_row - image_cell_height + offset.1).min(0);
    info!(
        "cursor: (r:{}, c:{}), image: (w:{}, h:{}), offset: (x:{}, y:{})",
        cur_row,
        cur_col,
        image_cell_width,
        image_cell_height,
        x_offset,
        y_offset
    );
    Ok((
        (x_offset as f32 * term_size.cell_width) as u32,
        (y_offset as f32 * term_size.cell_height) as u32,
    ))
}

#[instrument(skip(nvim))]
async fn process_req_start_hover(
    args: Vec<Value>, nvim: Neovim<NvimWriter>, session: Arc<NeovimSession>,
) -> anyhow::Result<u32> {
    if args.len() != 1 {
        bail!("hover expects 1 arguments, got {}", args.len());
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
        let st = std::time::Instant::now();
        let image =
            build_hover_doc_image(nvim.clone(), session.clone(), &md).await;
        let ed = std::time::Instant::now();
        info!("build hover doc image cost: {:?}", (ed - st).as_millis());
        match image {
            Ok((image, image_size)) => {
                let image =
                    IMAGE_MANAGER.lock().new_image_from_id_buffer(id, image);
                let (x, y) = {
                    let cfg = CONFIG.lock();
                    (cfg.hover.window.x_offset, cfg.hover.window.y_offset)
                };
                let (x, y) = image_offset_to_term(&nvim, image_size, (x, y))
                    .await
                    .unwrap();
                let writer = session.get_tty_writer(&nvim).await.unwrap();
                let mut writer = writer.lock().await;
                image.render_at(&mut writer, x, y).await.unwrap();
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

/// Expect name: "stop_hover"
#[instrument(skip(nvim))]
async fn process_req_stop_hover(
    args: Vec<Value>, nvim: Neovim<NvimWriter>, session: Arc<NeovimSession>,
) -> anyhow::Result<u32> {
    if args.len() != 1 {
        bail!("stop_hover expects 1 argument, got {}", args.len());
    }
    let id =
        NonZeroU32::try_from(args[0].as_u64().context("Expect u64")? as u32)?;
    tokio::spawn(async move {
        let image = IMAGE_MANAGER.lock().find_image(id);
        if let Some(image) = image {
            let writer = session.get_tty_writer(&nvim).await.unwrap();
            let mut writer = writer.lock().await;
            if let Err(e) = image.delete_image(&mut writer).await {
                nvim.err_writeln(&format!("Error deleting image: {}", e))
                    .await
                    .unwrap_or_else(|e| {
                        warn!("Error writing to nvim: {}", e);
                    });
            }
        }
    });
    Ok(u32::from(id))
}

#[derive(Debug)]
pub(crate) struct StartHoverReq;
#[derive(Debug)]
pub(crate) struct StopHoverReq;

#[async_trait]
impl NeovimService for StartHoverReq {
    #[instrument(skip(self, neovim))]
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

#[async_trait]
impl NeovimService for StopHoverReq {
    #[instrument(skip(self, neovim))]
    async fn call(
        &self, _name: String, args: Vec<Value>, neovim: Neovim<NvimWriter>,
        session: Arc<NeovimSession>,
    ) -> Result<Value, Value> {
        match process_req_stop_hover(args, neovim, session).await {
            Ok(v) => Ok(Value::from(v)),
            Err(e) => Err(Value::from(e.to_string())),
        }
    }
}
