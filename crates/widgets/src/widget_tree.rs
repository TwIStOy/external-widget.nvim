use std::sync::Arc;

use anyhow::{bail, Context};
use external_widget_core::RenderCtx;
use taffy::{AvailableSpace, NodeId, Point, TaffyTree, TraversePartialTree};

use crate::widget::Widget;

pub struct WidgetTree {
    root: Option<NodeId>,
    tree: TaffyTree<Arc<dyn Widget>>,
}

impl WidgetTree {
    pub fn new() -> Self {
        Self {
            root: None,
            tree: TaffyTree::new(),
        }
    }

    pub fn print_tree(&mut self) {
        if let Some(root) = &self.root {
            // must compute layout before printing
            self.tree
                .compute_layout_with_measure(
                    *root,
                    taffy::Size {
                        width: AvailableSpace::Definite(800.),
                        height: AvailableSpace::Definite(600.),
                    },
                    |known_dimensions, available_space, _, ctx| {
                        ctx.unwrap().measure(known_dimensions, available_space)
                    },
                )
                .unwrap();
            self.tree.print_tree(*root);
        }
    }

    pub fn new_leaf_with_context(
        &mut self, style: taffy::Style, widget: Arc<dyn Widget>,
    ) -> anyhow::Result<NodeId> {
        let node = self.tree.new_leaf_with_context(style, widget)?;
        Ok(node)
    }

    pub fn set_root(&mut self, root: NodeId) {
        self.root = Some(root);
    }

    pub fn add_child(
        &mut self, parent: NodeId, child: NodeId,
    ) -> anyhow::Result<()> {
        self.tree.add_child(parent, child)?;
        Ok(())
    }

    pub fn draw_from_root(&self, ctx: &RenderCtx) -> anyhow::Result<()> {
        if let Some(root) = &self.root {
            self.draw_node(*root, ctx, Point::ZERO)
        } else {
            bail!("no root node")
        }
    }

    fn draw_node(
        &self, node: NodeId, ctx: &RenderCtx, parent_abs_location: Point<f32>,
    ) -> anyhow::Result<()> {
        let layout = self.tree.layout(node)?;
        let node_ctx = self
            .tree
            .get_node_context(node)
            .context("no context in node")?;
        node_ctx.render(ctx, layout, parent_abs_location)?;
        for cid in self.tree.child_ids(node) {
            self.draw_node(cid, ctx, parent_abs_location + layout.location)?;
        }
        Ok(())
    }
}

impl Default for WidgetTree {
    fn default() -> Self {
        Self::new()
    }
}
