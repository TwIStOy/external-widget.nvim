use std::sync::Arc;

use external_widget_core::RenderCtx;
use taffy::Style;

use crate::widget::Widget;

#[derive(Debug)]
pub struct Row {
    children: Vec<Arc<dyn Widget>>,
}

impl Row {
    pub fn new(children: Vec<Arc<dyn Widget>>) -> Self {
        Self { children }
    }
}

impl Widget for Row {
    fn measure(
        &self, known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
    ) -> taffy::prelude::Size<f32> {
        let width_constraint = known_dimensions.width.unwrap_or({
            match available_space.width {
                taffy::AvailableSpace::Definite(width) => width,
                taffy::AvailableSpace::MinContent => 0.0,
                taffy::AvailableSpace::MaxContent => f32::INFINITY,
            }
        });
        taffy::Size::ZERO
    }

    fn register(
        self: Arc<Self>, tree: &mut crate::WidgetTree,
    ) -> anyhow::Result<taffy::prelude::NodeId> {
        let mut children = vec![];
        for child in &self.children {
            let child_id = child.clone().register(tree)?;
            children.push(child_id);
        }
        let id = tree.new_leaf_with_context(
            Style {
                display: taffy::Display::Flex,
                flex_direction: taffy::FlexDirection::Row,
                ..Default::default()
            },
            self,
        )?;
        for child_id in children {
            tree.add_child(id, child_id)?;
        }
        Ok(id)
    }

    fn render(
        &self, ctx: &RenderCtx, layout: &taffy::Layout,
        parent_abs_location: taffy::Point<f32>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
