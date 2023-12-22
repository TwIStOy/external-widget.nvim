use std::{cell::RefCell, fmt::Debug};

use skia_safe::textlayout::Paragraph;
use tracing::{instrument, trace};

use crate::{
    painting::{RectSize, RenderCtx, SpacePolicy},
    widgets::{
        widget::{LayoutElement, Widget, WidgetKey},
        BoxOptions,
    },
};

pub struct RichText {
    key: WidgetKey,
    paragraph: RefCell<Paragraph>,
    text: Option<String>,
}

impl RichText {
    /// Create a new RichText widget with the given paragraph.
    pub fn new_with_paragraph(
        paragraph: Paragraph, text: Option<String>,
    ) -> Self {
        Self {
            key: WidgetKey::next(),
            paragraph: RefCell::new(paragraph),
            text,
        }
    }
}

impl Debug for RichText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RichText")
            .field("text", &self.text)
            .finish()
    }
}

impl LayoutElement for RichText {
    fn style(&self) -> BoxOptions {
        Default::default()
    }

    fn compute_layout(
        &self, known_dimensions: RectSize<Option<f32>>,
        available_space: RectSize<SpacePolicy>,
    ) -> RectSize<f32> {
        let width_constraint = known_dimensions.width.unwrap_or({
            match available_space.width {
                SpacePolicy::Fixed(w) => w,
                SpacePolicy::Shrink => 0.,
                SpacePolicy::Expand => f32::INFINITY,
            }
        });
        let mut paragraph = self.paragraph.borrow_mut();
        paragraph.layout(width_constraint);
        trace!(
            "compute text {:?}, width_constraint: {}, long: {}",
            self.text,
            width_constraint,
            paragraph.longest_line()
        );
        RectSize {
            width: paragraph.longest_line().ceil(),
            height: paragraph.height().ceil(),
        }
    }
}

impl Widget for RichText {
    fn key(&self) -> WidgetKey {
        self.key
    }

    #[instrument(skip(self, context))]
    fn paint(&self, context: &mut RenderCtx<'_>) -> anyhow::Result<()> {
        let canvas = context.render.canvas();
        let mut paragraph = self.paragraph.borrow_mut();
        trace!(
            "paint text {:?}, width: {}, l: {:?}",
            self.text,
            context.size.width,
            context.top_left_location
        );
        // confirm that the paragraph is laid out
        paragraph.layout(context.size.width);
        trace!("long?: {}", paragraph.longest_line());
        paragraph.paint(canvas, context.top_left_location);
        Ok(())
    }
}
