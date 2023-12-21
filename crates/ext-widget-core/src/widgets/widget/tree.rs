use std::rc::Rc;

use anyhow::{bail, Context};
use taffy::{AvailableSpace, NodeId, TaffyTree};
use tracing::trace;

use crate::painting::RectSize;

use super::{Widget, WidgetExt};

/// Widget tree
pub struct WidgetTree {
    root: Option<NodeId>,
    inner: TaffyTree<Rc<dyn Widget>>,
    last_size: Option<(f32, f32)>,
}

impl WidgetTree {
    pub fn new() -> Self {
        Self {
            root: None,
            inner: TaffyTree::new(),
            last_size: None,
        }
    }

    /// Compute and update layout of all nodes from root.
    pub fn compute_layout(
        &mut self, width: f32, height: f32,
    ) -> anyhow::Result<()> {
        if self.root.is_none() {
            bail!("root is not set")
        }
        let root = self.root.unwrap();
        if self.last_size == Some((width, height)) && !self.inner.dirty(root)? {
            return Ok(());
        }
        self.last_size = Some((width, height));
        self.inner.compute_layout_with_measure(
            root,
            taffy::Size {
                width: AvailableSpace::Definite(width),
                height: AvailableSpace::Definite(height),
            },
            |known_dimensions, available_space, _, ctx| {
                trace!(
                    "compute layout: {:?} {:?}",
                    known_dimensions,
                    available_space
                );
                ctx.unwrap()
                    .compute_layout(
                        RectSize {
                            width: known_dimensions.width,
                            height: known_dimensions.height,
                        },
                        RectSize {
                            width: match available_space.width {
                                AvailableSpace::Definite(v) => v,
                                _ => unreachable!(),
                            },
                            height: match available_space.height {
                                AvailableSpace::Definite(v) => v,
                                _ => unreachable!(),
                            },
                        },
                    )
                    .into()
            },
        )?;

        Ok(())
    }

    /// Mount a widget into the widget tree, and set it as the root.
    pub fn new_root(
        &mut self, widget: Rc<dyn Widget>,
    ) -> anyhow::Result<NodeId> {
        let node = self.new_leaf(widget)?;
        self.root = Some(node);
        Ok(node)
    }

    /// Mount a widget (and all recursive children) into the widget tree.
    pub fn new_leaf(
        &mut self, widget: Rc<dyn Widget>,
    ) -> anyhow::Result<NodeId> {
        let mut style: taffy::Style = widget.style().into();
        style.display = taffy::Display::Flex;
        let node = self.inner.new_leaf_with_context(style, widget.clone())?;
        for child in widget.children() {
            let cid = self.new_leaf(child.clone())?;
            self.inner.add_child(node, cid)?;
        }
        Ok(node)
    }

    pub fn debug_tree(&self) -> anyhow::Result<Vec<String>> {
        if self.root.is_none() {
            bail!("root is not set")
        }
        let root = self.root.unwrap();

        let mut lines = vec![];
        let mut lasts = vec![];

        self.inner
            .get_node_context(root)
            .context("No context?")?
            .debug_tree(&mut lasts, &mut lines);

        Ok(lines)
    }

    fn dfs<F>(&self, node: NodeId, f: &mut F) -> anyhow::Result<()>
    where
        F: FnMut(NodeId),
    {
        f(node);
        for child in self.inner.children(node)? {
            self.dfs(child, f)?;
        }
        Ok(())
    }

    fn bfs<F>(&self, node: NodeId, f: &mut F) -> anyhow::Result<()>
    where
        F: FnMut(NodeId),
    {
        let mut queue = vec![node];
        while let Some(node) = queue.pop() {
            f(node);
            for child in self.inner.children(node)? {
                queue.push(child);
            }
        }
        Ok(())
    }
}
