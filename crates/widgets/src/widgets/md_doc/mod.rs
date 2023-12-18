use std::sync::Arc;

use comrak::{parse_document, Arena, Options};
use external_widget_core::Widget;

mod converter;

pub fn md2markup(md: &str) {
    let arena = Arena::new();
    let opts = Options {
        ..Default::default()
    };
    let root = parse_document(&arena, md, &opts);
    let mut converter = converter::Converter::new();
    let res = converter.visit_node(root).unwrap();

    res.print_element();
}
