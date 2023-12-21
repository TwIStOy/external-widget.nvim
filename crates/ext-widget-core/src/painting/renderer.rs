use std::fmt::Debug;

use anyhow::Context;
use base64::Engine;
use skia_safe::{surfaces, Canvas, EncodedImageFormat, Surface};

use super::{Location, RectSize};

/// A renderer that can paint widgets.
#[derive(Debug)]
pub struct Renderer {
    surface: Surface,
}

impl Renderer {
    pub fn new() -> anyhow::Result<Self> {
        let surface = surfaces::raster_n32_premul((100, 100)).unwrap();
        Ok(Self { surface })
    }

    pub fn canvas(&mut self) -> &Canvas {
        self.surface.canvas()
    }

    /// Take snapshot of current canvas, encode it to png, then encode it to
    /// base64 format.
    pub fn snapshot_png(&mut self) -> anyhow::Result<String> {
        let image = self.surface.image_snapshot();
        let data = image
            .encode(
                &mut self.surface.direct_context(),
                EncodedImageFormat::PNG,
                None,
            )
            .context("Failed to encode png")?;
        let encoded =
            base64::engine::general_purpose::STANDARD.encode(data.as_bytes());
        Ok(encoded)
    }
}

#[derive(Debug)]
pub struct RenderCtx<'a> {
    pub render: &'a mut Renderer,
    /// Absolute location of this widget.
    pub top_left_location: Location,
    /// Real size of the this widget.
    pub size: RectSize<f32>,
    /// Content size of this widget.
    pub content_size: RectSize<f32>,
}