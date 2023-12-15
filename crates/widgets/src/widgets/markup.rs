use std::sync::Arc;

use taffy::{Size, Style};

use crate::widget::{Widget, WidgetTree};

#[derive(Debug)]
pub struct MarkupParagraph {
    markup: String,
    layout: pango::Layout,
}

impl MarkupParagraph {
    pub fn new(ctx: &cairo::Context, markup: String) -> Self {
        let layout = pangocairo::create_layout(ctx);
        Self { markup, layout }
    }
}

impl Widget for MarkupParagraph {
    fn register(self: Arc<Self>, tree: &mut WidgetTree) -> anyhow::Result<()> {
        tree.new_leaf_with_context(Style::default(), self)?;
        Ok(())
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
            self.layout.set_width(-1);
        } else {
            self.layout
                .set_width(width_constraint as i32 * pango::SCALE);
        }
        let (width, height) = self.layout.pixel_size();
        Size {
            width: width as f32,
            height: height as f32,
        }
    }

    fn render(
        &self, ctx: &cairo::Context, layout: &taffy::Layout,
        _parent_abs_location: taffy::Point<f32>,
    ) {
        layout.location;
        // layout.position
    }
}
