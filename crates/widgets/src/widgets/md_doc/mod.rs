use std::{collections::HashMap, rc::Rc};

use comrak::{parse_document, Arena, Options};
use external_widget_core::{
    nvim::HighlightDefinition, pango::MarkupProperties, print_element_marker,
    Widget,
};
use taffy::Style;

mod codeblock;
mod converter;

#[derive(Debug, Clone)]
pub struct MdDoc {
    root_widget: Rc<dyn Widget>,
    opts: MdDocOpts,
}

#[derive(Debug, Clone)]
pub struct MdDocOpts {
    pub md: String,
    pub highlights: HashMap<String, HighlightDefinition>,
    pub normal_font: String,
    pub mono_font: String,
    pub font_size: String,
}

impl MdDoc {
    pub fn new(opts: MdDocOpts) -> anyhow::Result<Self> {
        let arena = Arena::new();
        let parse_opts = Options {
            ..Default::default()
        };
        let root = parse_document(&arena, &opts.md, &parse_opts);
        let mut converter = converter::Converter::new(&opts);
        let root_widget = converter.visit_node(root)?;
        Ok(Self { opts, root_widget })
    }
}

impl Widget for MdDoc {
    fn register(
        self: Rc<Self>, tree: &mut external_widget_core::WidgetTree,
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
        println!("MdDoc {:?}", self.opts.md);
        lasts.push(true);
        self.root_widget.print_element_impl(lasts);
        lasts.pop();
    }
}
