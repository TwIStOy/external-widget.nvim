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

    fn print_element(&self) {
        let mut lasts = vec![];
        self.print_element_impl(&mut lasts);
    }

    fn print_element_impl(&self, lasts: &mut Vec<bool>);
}

const TREE_MARKER_LAST: &str = "└── ";
const TREE_MARKER_MIDDLE: &str = "├── ";
const TREE_MARKER_VERTICAL: &str = "│   ";

pub fn print_element_marker(lasts: &[bool]) {
    for (i, last) in lasts.iter().enumerate() {
        if i == lasts.len() - 1 {
            if *last {
                print!("{}", TREE_MARKER_LAST);
            } else {
                print!("{}", TREE_MARKER_MIDDLE);
            }
        } else if *last {
            print!("{}", " ".repeat(4));
        } else {
            print!("{}", TREE_MARKER_VERTICAL);
        }
    }
}
