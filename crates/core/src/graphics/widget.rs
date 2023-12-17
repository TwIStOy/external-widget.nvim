use std::{fmt::Debug, sync::Arc};

use taffy::{prelude::*, Point};

use super::{MeasureCtx, RenderCtx, WidgetTree};

pub trait Widget: Debug {
    /// Measure the widget, returns the size of the widget.
    fn measure(
        &self, _ctx: &MeasureCtx, _known_dimensions: taffy::Size<Option<f32>>,
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

    fn print_element(&self, last: bool, depth: usize);
}

const TREE_MARKER_LAST: &str = "└── ";
const TREE_MARKER_MIDDLE: &str = "├── ";

pub fn print_element_marker(last: bool, depth: usize) {
    let indent = " ".repeat((depth.max(1) - 1) * 4);
    if depth >= 1 {
        print!(
            "{}{}",
            indent,
            if last {
                TREE_MARKER_LAST
            } else {
                TREE_MARKER_MIDDLE
            }
        );
    }
}
