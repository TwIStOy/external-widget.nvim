use std::rc::Rc;

use skia_safe::Paint;

use crate::{
    painting::{BoxDecoration, RectSize, SpacePolicy},
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

impl Container {
    pub fn new(decoration: BoxDecoration, options: BoxOptions) -> Self {
        Self {
            key: WidgetKey::next(),
            child: None,
            decoration,
            options,
        }
    }
}

impl LayoutElement for Container {
    fn style(&self) -> BoxOptions {
        self.options.clone()
    }

    fn compute_layout(
        &self, known_dimensions: RectSize<Option<f32>>,
        available_space: RectSize<SpacePolicy>,
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
        let mut fill_paint = Paint::default();
        let mut stroke_paint = Paint::default();
        fill_paint.set_style(skia_safe::paint::Style::Fill);
        stroke_paint.set_style(skia_safe::paint::Style::Stroke);

        // set color
        let rect = skia_safe::Rect::from_point_and_size(
            context.top_left_location,
            context.size,
        );

        if self.decoration.border.width > 0.0 {
            stroke_paint.set_stroke_width(self.decoration.border.width);
        }
        let radius = match self.decoration.border.radius {
            crate::painting::FlexibleLength::Fixed(l) => l,
            crate::painting::FlexibleLength::Percent(p) => {
                context.size.width * p
            }
        };
        if self.decoration.color.alpha > 0 {
            let color: skia_safe::Color = self.decoration.color.into();
            fill_paint.set_color(color);
        }

        canvas
            .draw_round_rect(rect, radius, radius, &fill_paint)
            .draw_round_rect(rect, radius, radius, &stroke_paint);

        Ok(())
    }

    fn children(&self) -> Vec<Rc<dyn Widget>> {
        if let Some(c) = &self.child {
            vec![c.clone()]
        } else {
            vec![]
        }
    }
}

impl Container {
    fn should_paint(&self) -> bool {
        self.decoration.color.alpha > 0
            || (self.decoration.border.color.alpha > 0
                && self.decoration.border.width > 0.0)
    }
}
