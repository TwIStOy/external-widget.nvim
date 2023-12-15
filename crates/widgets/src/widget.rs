use std::sync::Arc;

use anyhow::Context;
use taffy::{prelude::*, Point};

pub trait Widget {
    /// Measure the widget, returns the size of the widget.
    fn measure(
        &self, known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
    ) -> Size<f32>;

    /// Resiger the widget into the tree.
    fn register(self: Arc<Self>, tree: &mut WidgetTree) -> anyhow::Result<()>;

    /// Render the widget.
    fn render(
        &self, ctx: &cairo::Context, layout: &taffy::Layout,
        parent_abs_location: Point<f32>,
    );
}

pub struct WidgetTree {
    // root: NodeId,
    tree: TaffyTree<Arc<dyn Widget>>,
}

impl WidgetTree {
    pub fn print_tree(&self) {
        // self.tree.print_tree(root);
    }

    pub fn new_leaf_with_context(
        &mut self, style: Style, widget: Arc<dyn Widget>,
    ) -> anyhow::Result<NodeId> {
        let node = self.tree.new_leaf_with_context(style, widget)?;
        Ok(node)
    }

    pub fn draw_from_root(
        &self, ctx: &cairo::Context, root: &NodeId,
    ) -> anyhow::Result<()> {
        self.draw_node(root, ctx, Point::ZERO)
    }

    fn draw_node(
        &self, node: &NodeId, ctx: &cairo::Context,
        parent_abs_location: Point<f32>,
    ) -> anyhow::Result<()> {
        let layout = self.tree.layout(node.clone())?;
        let node_ctx = self
            .tree
            .get_node_context(node.clone())
            .context("no context in node")?;
        node_ctx.render(ctx, layout, parent_abs_location);

        Ok(())
    }
}

pub fn test(tree: &mut WidgetTree, root: NodeId) {
    // tree.children(root);
    // tree.child_ids(parent_node_id)
    // tree.layout(node);
    // tree.compute_layout_with_measure(root, available_space, measure_function);
}
