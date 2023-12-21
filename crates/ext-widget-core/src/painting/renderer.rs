use std::fmt::Debug;

use skia_safe::{surfaces, Canvas, Surface};

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
