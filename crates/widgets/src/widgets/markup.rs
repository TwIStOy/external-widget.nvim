use std::{fmt::Display, rc::Rc, sync::Arc};

use external_widget_core::{
    print_element_marker, MeasureCtx, RenderCtx, Widget, WidgetTree,
};
use taffy::{NodeId, Size, Style};

#[derive(Debug)]
pub struct MarkupParagraph {
    markup: String,
}

impl MarkupParagraph {
    pub fn new(markup: String) -> Self {
        Self { markup }
    }
}

impl Display for MarkupParagraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.markup)
    }
}

impl Widget for MarkupParagraph {
    fn register(
        self: Rc<Self>, tree: &mut WidgetTree,
    ) -> anyhow::Result<NodeId> {
        tree.new_leaf_with_context(Style::default(), self)
    }

    fn measure(
        &self, ctx: &MeasureCtx, known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
    ) -> Size<f32> {
        let width_constraint = known_dimensions.width.unwrap_or_else(|| {
            match available_space.width {
                taffy::AvailableSpace::Definite(width) => width,
                taffy::AvailableSpace::MinContent => 0.0,
                taffy::AvailableSpace::MaxContent => -1.0,
            }
        });
        let layout = pangocairo::create_layout(ctx.inner());
        layout.set_markup(&self.markup);
        if width_constraint < 0.0 {
            layout.set_width(-1);
        } else {
            layout.set_width(width_constraint as i32 * pango::SCALE);
        }
        let (width, height) = layout.pixel_size();
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
        let pango_layout = pangocairo::create_layout(ctx.inner());
        pango_layout.set_markup(&self.markup);
        pangocairo::show_layout(ctx.inner(), &pango_layout);
        Ok(())
    }

    fn print_element_impl(&self, lasts: &mut Vec<bool>) {
        print_element_marker(lasts);
        println!("MarkupParagraph: {}", self.markup);
    }
}
