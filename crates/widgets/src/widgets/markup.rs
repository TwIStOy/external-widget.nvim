use std::{fmt::Display, sync::Arc};

use external_widget_core::RenderCtx;
use taffy::{NodeId, Size, Style};

use crate::{widget::Widget, WidgetTree};

#[derive(Debug)]
pub struct MarkupParagraph {
    markup: String,
    pango_layout: pango::Layout,
}

impl MarkupParagraph {
    pub fn new(ctx: &cairo::Context, markup: String) -> Self {
        let layout = pangocairo::create_layout(ctx);
        layout.set_markup(&markup);
        Self {
            markup,
            pango_layout: layout,
        }
    }
}

impl Display for MarkupParagraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.markup)
    }
}

impl Widget for MarkupParagraph {
    fn register(
        self: Arc<Self>, tree: &mut WidgetTree,
    ) -> anyhow::Result<NodeId> {
        tree.new_leaf_with_context(Style::default(), self)
    }

    fn measure(
        &self, known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
    ) -> Size<f32> {
        let width_constraint = known_dimensions.width.unwrap_or_else(|| {
            match available_space.width {
                taffy::AvailableSpace::Definite(width) => width,
                taffy::AvailableSpace::MinContent => 0.0,
                taffy::AvailableSpace::MaxContent => -1.0,
            }
        });
        if width_constraint < 0.0 {
            self.pango_layout.set_width(-1);
        } else {
            self.pango_layout
                .set_width(width_constraint as i32 * pango::SCALE);
        }
        let (width, height) = self.pango_layout.pixel_size();
        Size {
            width: width as f32,
            height: height as f32,
        }
    }

    fn render(
        &self, ctx: &RenderCtx, layout: &taffy::Layout,
        parent_abs_location: taffy::Point<f32>,
    ) -> anyhow::Result<()> {
        let location = parent_abs_location + layout.location;
        ctx.move_to(location.x as f64, location.y as f64);
        pangocairo::show_layout(ctx.inner(), &self.pango_layout);
        Ok(())
    }
}
