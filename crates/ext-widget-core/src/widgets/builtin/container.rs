use std::rc::Rc;

use skia_safe::Paint;

use crate::{
    painting::{BoxDecoration, RectSize},
    widgets::{
        widget::{LayoutElement, Widget, WidgetKey},
        BoxOptions,
    },
};

#[derive(Debug)]
pub struct Container {
    pub key: WidgetKey,
    pub child: Option<Rc<dyn Widget>>,
    pub decoration: BoxDecoration,
    pub options: BoxOptions,
}

impl LayoutElement for Container {
    fn style(&self) -> BoxOptions {
        self.options.clone()
    }

    fn compute_layout(
        &self, known_dimensions: RectSize<Option<f32>>,
        available_space: RectSize<f32>,
    ) -> RectSize<f32> {
        if let Some(c) = &self.child {
            c.compute_layout(known_dimensions, available_space)
        } else {
            RectSize::default()
        }
    }
}

impl Widget for Container {
    fn key(&self) -> crate::widgets::widget::WidgetKey {
        self.key
    }

    fn paint(
        &self, context: &mut crate::painting::RenderCtx<'_>,
    ) -> anyhow::Result<()> {
        if !self.should_paint() {
            return Ok(());
        }

        let canvas = context.render.canvas();
        let mut paint = Paint::default();
        // set color
        let rect = skia_safe::Rect::from_point_and_size(
            context.top_left_location,
            context.size,
        );
        let mut rrect = skia_safe::RRect::new_rect(rect);

        if self.decoration.border.width > 0.0 {
            paint.set_stroke_width(self.decoration.border.width);
        }
        let radius = match self.decoration.border.radius {
            crate::painting::FlexibleLength::Fixed(l) => l,
            crate::painting::FlexibleLength::Percent(p) => {
                context.size.width * p
            }
        };
        if radius > 0.0 {
            rrect.set_rect_xy(rect, radius, radius);
        }
        if self.decoration.color.alpha > 0 {
            let color: skia_safe::Color = self.decoration.color.into();
            paint.set_color(color);
            canvas.draw_rrect(rrect, &paint);
        }

        canvas.draw_rect(rect, &paint);

        Ok(())
    }
}

impl Container {
    fn should_paint(&self) -> bool {
        self.decoration.color.alpha > 0
            || (self.decoration.border.color.alpha > 0
                && self.decoration.border.width > 0.0)
    }
}