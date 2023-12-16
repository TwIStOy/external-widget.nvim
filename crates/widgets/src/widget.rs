use std::{fmt::Debug, sync::Arc};

use external_widget_core::RenderCtx;
use taffy::{prelude::*, Point};

use crate::widget_tree::WidgetTree;

pub trait Widget: Debug {
    /// Measure the widget, returns the size of the widget.
    fn measure(
        &self, _known_dimensions: taffy::Size<Option<f32>>,
        _available_space: taffy::Size<taffy::AvailableSpace>,
    ) -> Size<f32> {
        Size::ZERO
    }

    /// Resiger the widget into the tree.
    fn register(
        self: Arc<Self>, tree: &mut WidgetTree,
    ) -> anyhow::Result<NodeId>;

    /// Render the widget.
    fn render(
        &self, ctx: &RenderCtx, layout: &taffy::Layout,
        parent_abs_location: Point<f32>,
    ) -> anyhow::Result<()>;
}
