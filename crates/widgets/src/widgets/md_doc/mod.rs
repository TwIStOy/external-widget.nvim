use std::sync::Arc;

use comrak::{parse_document, Arena, Options};
use external_widget_core::{
    pango::MarkupProperties, print_element_marker, Widget,
};
use taffy::Style;

mod converter;

#[derive(Debug, Clone)]
pub struct MdDoc {
    md: String,
    root_widget: Arc<dyn Widget>,
}

impl MdDoc {
    pub fn new(md: String) -> anyhow::Result<Self> {
        let arena = Arena::new();
        let opts = Options {
            ..Default::default()
        };
        let root = parse_document(&arena, &md, &opts);
        let mut converter = converter::Converter::new(MarkupProperties::new());
        let root_widget = converter.visit_node(root)?;
        Ok(Self { md, root_widget })
    }
}

impl Widget for MdDoc {
    fn register(
        self: std::sync::Arc<Self>, tree: &mut external_widget_core::WidgetTree,
    ) -> anyhow::Result<taffy::prelude::NodeId> {
        let style = Style {
            ..Default::default()
        };
        let id = tree.new_leaf_with_context(style, self.clone())?;
        let root_id = self.root_widget.clone().register(tree)?;
        tree.add_child(id, root_id)?;
        Ok(id)
    }

    fn render(
        &self, _ctx: &external_widget_core::RenderCtx, _layout: &taffy::Layout,
        _parent_abs_location: taffy::Point<f32>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn print_element_impl(&self, lasts: &mut Vec<bool>) {
        print_element_marker(lasts);
        println!("MdDoc {:?}", self.md);
        lasts.push(true);
        self.root_widget.print_element_impl(lasts);
        lasts.pop();
    }
}
