use std::sync::Arc;

use external_widget_core::{
    print_element_marker, Color, RenderCtx, Widget, WidgetTree,
};
use taffy::prelude::*;

#[derive(Debug)]
pub struct Bar {
    width: Option<LengthPercentage>,
    height: Option<LengthPercentage>,
    color: Color,
}

impl Bar {
    pub fn new() -> Self {
        Self {
            width: None,
            height: None,
            color: Color::from_u32(0),
        }
    }

    pub fn width(mut self, width: LengthPercentage) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: LengthPercentage) -> Self {
        self.height = Some(height);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Default for Bar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Bar {
    fn register(
        self: Arc<Self>, tree: &mut WidgetTree,
    ) -> anyhow::Result<NodeId> {
        let style = Style {
            size: Size {
                width: self.width.map(|x| x.into()).unwrap_or(Dimension::Auto),
                height: self
                    .height
                    .map(|x| x.into())
                    .unwrap_or(Dimension::Auto),
            },
            ..Default::default()
        };
        let id = tree.new_leaf_with_context(style, self.clone())?;
        Ok(id)
    }

    fn render(
        &self, ctx: &RenderCtx, layout: &taffy::Layout,
        parent_abs_location: taffy::Point<f32>,
    ) -> anyhow::Result<()> {
        let location = parent_abs_location + layout.location;
        let size = layout.size;
        ctx.rectangle(location.x, location.y, size.width, size.height);
        ctx.set_color(&self.color);
        ctx.fill()?;
        Ok(())
    }

    fn print_element_impl(&self, lasts: &mut Vec<bool>) {
        print_element_marker(lasts);
        println!("Bar");
    }
}
