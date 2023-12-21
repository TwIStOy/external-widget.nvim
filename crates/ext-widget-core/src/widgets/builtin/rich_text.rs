use std::cell::RefCell;

use skia_safe::textlayout::Paragraph;

use crate::{
    painting::{RectSize, RenderCtx},
    widgets::{
        widget::{LayoutElement, Widget, WidgetKey},
        BoxOptions,
    },
};

#[derive(Debug)]
pub struct RichText {
    key: WidgetKey,
    paragraph: RefCell<Paragraph>,
}

impl RichText {
    /// Create a new RichText widget with the given paragraph.
    pub fn new_with_paragraph(paragraph: Paragraph) -> Self {
        Self {
            key: WidgetKey::next(),
            paragraph: RefCell::new(paragraph),
        }
    }
}

impl LayoutElement for RichText {
    fn style(&self) -> BoxOptions {
        Default::default()
    }

    fn compute_layout(
        &self, known_dimensions: RectSize<Option<f32>>,
        available_space: RectSize<f32>,
    ) -> RectSize<f32> {
        let width_constraint =
            known_dimensions.width.unwrap_or(available_space.width);
        let mut paragraph = self.paragraph.borrow_mut();
        paragraph.layout(width_constraint);
        RectSize {
            width: paragraph.longest_line(),
            height: paragraph.height(),
        }
    }
}

impl Widget for RichText {
    fn key(&self) -> WidgetKey {
        self.key
    }

    fn paint(&self, render: &mut RenderCtx<'_>) -> anyhow::Result<()> {
        let canvas = render.render.canvas();
        let mut paragraph = self.paragraph.borrow_mut();
        // confirm that the paragraph is laid out
        paragraph.layout(render.size.width);
        paragraph.paint(canvas, render.top_left_location);
        Ok(())
    }
}
