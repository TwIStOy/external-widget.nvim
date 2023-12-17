use std::sync::Arc;

use external_widget_core::{
    print_element_marker, Color, MeasureCtx, RenderCtx, Widget, WidgetTree,
};
use taffy::prelude::*;

use crate::support::BoxConstraints;

#[derive(Debug)]
pub struct ContainerBorderStyle {
    pub color: Color,
    pub width: f32,
}

#[derive(Debug)]
pub struct Container {
    pub child: Option<Arc<dyn Widget>>,
    pub constraints: BoxConstraints,
    pub background: Option<Color>,
    pub border: Option<ContainerBorderStyle>,
    pub corner_radius: Option<LengthPercentage>,
}

impl Container {
    pub fn new(child: Arc<dyn Widget>) -> Self {
        Self {
            child: Some(child),
            background: None,
            border: None,
            corner_radius: None,
            constraints: BoxConstraints::default(),
        }
    }
}

impl Widget for Container {
    fn measure(
        &self, ctx: &MeasureCtx, known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        if let Some(c) = &self.child {
            c.measure(ctx, known_dimensions, available_space)
        } else {
            taffy::Size::ZERO
        }
    }

    fn register(
        self: std::sync::Arc<Self>, tree: &mut WidgetTree,
    ) -> anyhow::Result<NodeId> {
        let style = Style {
            min_size: Size {
                width: (self.constraints.min_width),
                height: (self.constraints.min_height),
            },
            ..Default::default()
        };
        let id = tree.new_leaf_with_context(style, self.clone())?;
        if let Some(child) = &self.child {
            let child_id = child.clone().register(tree)?;
            tree.add_child(id, child_id)?;
        }
        Ok(id)
    }

    fn render(
        &self, ctx: &RenderCtx, layout: &taffy::Layout,
        parent_abs_location: taffy::Point<f32>,
    ) -> anyhow::Result<()> {
        let abs_location = parent_abs_location + layout.location;

        let x = abs_location.x;
        let y = abs_location.y;
        let width = layout.size.width;
        let height = layout.size.height;
        let radius =
            match self.corner_radius.unwrap_or(LengthPercentage::Length(0.0)) {
                LengthPercentage::Length(l) => l,
                LengthPercentage::Percent(p) => p * width.min(height),
            };
        let degree = std::f64::consts::PI / 180.0;

        ctx.move_to(x as f64, y as f64);

        ctx.save()?;
        ctx.new_sub_path();
        // rounded rectangle
        ctx.arc(x + width - radius, y + radius, radius, -90.0 * degree, 0.);
        ctx.arc(
            x + width - radius,
            y + height - radius,
            radius,
            0,
            90.0 * degree,
        );
        ctx.arc(
            x + radius,
            y + height - radius,
            radius,
            90. * degree,
            180. * degree,
        );
        ctx.arc(x + radius, y + radius, radius, 180. * degree, 270. * degree);
        ctx.close_path();

        // draw background
        if let Some(background) = &self.background {
            ctx.set_color(background);
            ctx.fill_preserve()?;
        }

        // draw border
        if let Some(border) = &self.border {
            ctx.set_color(&border.color);
            ctx.set_line_width(border.width);
            ctx.stroke()?;
        }
        ctx.restore()?;

        Ok(())
    }

    fn print_element(&self, last: bool, depth: usize) {
        print_element_marker(last, depth);
        println!("Container");
        if let Some(c) = &self.child {
            c.print_element(true, depth + 1);
        }
    }
}
