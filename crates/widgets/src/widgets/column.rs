use std::sync::Arc;

use external_widget_core::{
    print_element_marker, MeasureCtx, RenderCtx, Widget, WidgetTree,
};
use taffy::Style;

#[derive(Debug)]
pub struct Column {
    children: Vec<Arc<dyn Widget>>,
}

impl Column {
    pub fn new(children: Vec<Arc<dyn Widget>>) -> Self {
        Self { children }
    }
}

impl Widget for Column {
    fn measure(
        &self, _ctx: &MeasureCtx, known_dimensions: taffy::Size<Option<f32>>,
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
        self: Arc<Self>, tree: &mut WidgetTree,
    ) -> anyhow::Result<taffy::prelude::NodeId> {
        let mut children = vec![];
        for child in &self.children {
            let child_id = child.clone().register(tree)?;
            children.push(child_id);
        }
        let id = tree.new_leaf_with_context(
            Style {
                display: taffy::Display::Flex,
                flex_direction: taffy::FlexDirection::Column,
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

    fn print_element(&self, last: bool, depth: usize) {
        print_element_marker(last, depth);
        println!("Column");
        for (i, child) in self.children.iter().enumerate() {
            child.print_element(i == self.children.len() - 1, depth + 1);
        }
    }
}
