use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::{bail, Context};
use taffy::{AvailableSpace, NodeId, TaffyTree, TraversePartialTree};
use tracing::{instrument, trace};

use crate::painting::{Location, RectSize, RenderCtx, Renderer};

use super::{Widget, WidgetExt, WidgetKey};

/// Widget tree
pub struct WidgetTree {
    root: Option<NodeId>,
    inner: TaffyTree<Rc<dyn Widget>>,
    last_size: Option<(f32, f32)>,
    relation: HashMap<WidgetKey, NodeId>,
}

impl WidgetTree {
    pub fn new() -> Self {
        Self {
            root: None,
            inner: TaffyTree::new(),
            last_size: None,
            relation: HashMap::new(),
        }
    }

    /// Compute and update layout of all nodes from root.
    #[instrument(skip(self))]
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
                let ctx = ctx.unwrap();
                trace!(
                    "{} compute layout: {:?} {:?}",
                    ctx.type_name(),
                    known_dimensions,
                    available_space
                );
                ctx.compute_layout(
                    RectSize {
                        width: known_dimensions.width,
                        height: known_dimensions.height,
                    },
                    RectSize {
                        width: available_space.width.into(),
                        height: available_space.height.into(),
                    },
                )
                .into()
            },
        )?;

        Ok(())
    }

    /// Paint all nodes in the widget tree.
    pub fn paint(&self, renderer: Rc<RefCell<Renderer>>) -> anyhow::Result<()> {
        if self.root.is_none() {
            bail!("root is not set")
        }
        let root = self.root.unwrap();
        self.paint_node(root, renderer, Location { x: 0., y: 0. })
    }

    fn paint_node(
        &self, node: NodeId, renderer: Rc<RefCell<Renderer>>,
        top_left: Location,
    ) -> anyhow::Result<()> {
        let widget = self.inner.get_node_context(node).unwrap();
        let layout = self.inner.layout(node)?;
        let top_left = top_left + layout.location;
        {
            let mut ctx = RenderCtx {
                render: &mut renderer.borrow_mut(),
                top_left_location: top_left,
                size: layout.size.into(),
                content_size: layout.content_size.into(),
            };
            widget.paint(&mut ctx)?;
        }
        for c in self.inner.child_ids(node) {
            self.paint_node(c, renderer.clone(), top_left)?;
        }
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
    #[instrument(skip(self))]
    pub fn new_leaf(
        &mut self, widget: Rc<dyn Widget>,
    ) -> anyhow::Result<NodeId> {
        let mut style: taffy::Style = widget.style().into();
        style.display = taffy::Display::Flex;
        let node = self.inner.new_leaf_with_context(style, widget.clone())?;
        self.relation.insert(widget.key(), node);
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

        self.debug_tree_impl(root, &mut lasts, &mut lines)?;

        Ok(lines)
    }

    fn debug_tree_impl(
        &self, node: NodeId, lasts: &mut Vec<bool>, lines: &mut Vec<String>,
    ) -> anyhow::Result<()> {
        let widget = self.inner.get_node_context(node).context("No widget?")?;
        let layout = self.inner.layout(node).context("layout not found")?;
        let info = format!(
            "[x: {x} y: {y} w: {width} h: {height} content_w: \
            {content_width:<4} content_h: {content_height:<4}",
            x = layout.location.x,
            y = layout.location.y,
            width = layout.size.width,
            height = layout.size.height,
            content_width = layout.content_size.width,
            content_height = layout.content_size.height,
        );
        widget.debug_tree(info, lasts, lines);
        for (i, child) in self.inner.child_ids(node).enumerate() {
            let last = i == self.inner.child_count(node) - 1;
            lasts.push(last);
            self.debug_tree_impl(child, lasts, lines)?;
            lasts.pop();
        }
        Ok(())
    }

    fn debug_layout_info(&self, key: WidgetKey) -> anyhow::Result<String> {
        let node = self.relation.get(&key).context("No such widget")?;
        let layout = self.inner.layout(*node).context("layout not found")?;
        Ok(format!("{:?}", layout))
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

impl Default for WidgetTree {
    fn default() -> Self {
        Self::new()
    }
}
