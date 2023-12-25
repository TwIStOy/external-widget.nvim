mod nvim;

pub use nvim::{start_embed_nvim, EmbedNvim};

use std::{cell::RefCell, rc::Rc};

use tracing::trace;

use crate::{
    painting::Renderer,
    widgets::{Widget, WidgetTree},
};

pub fn render_widget_to_file(widget: Rc<dyn Widget>, filepath: &str) {
    let renderer = Rc::new(RefCell::new(Renderer::new(1000, 1000).unwrap()));
    let mut tree = WidgetTree::new();
    tree.new_root(widget).unwrap();
    tree.compute_layout(1000., 1000.).unwrap();

    let lines = tree.debug_tree().unwrap();
    trace!("\n{}", lines.join("\n"));
    tree.paint(renderer.clone()).unwrap();

    let png = renderer.borrow_mut().snapshot_png_raw().unwrap();
    std::fs::write(filepath, png).unwrap();
}
