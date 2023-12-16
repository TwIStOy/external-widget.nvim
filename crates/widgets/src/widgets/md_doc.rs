use std::sync::Arc;

use comrak::{nodes::AstNode, parse_document, Arena, Options};
use external_widget_core::pango::MarkupSpanStack;

use crate::{Container, Row, Widget};

#[derive(Debug, Clone)]
pub struct MdDocument {
    md: String,
}

// impl Widget for MdDocument {
//     fn layout(&mut self, _bc: &BoxConstraints) -> Dimension {
//         Dimension::new(0.0, 0.0)
//     }

//     fn draw(&mut self, _ctx: &mut RenderCtx) {
//         todo!()
//     }
// }

pub fn md2markup(md: &str) {
    let arena = Arena::new();
    let opts = Options {
        ..Default::default()
    };
    let root = parse_document(&arena, md, &opts);
    let mut stack = MarkupSpanStack::new();
    visit_node(root, &mut stack, 0);
}

pub fn visit_node<'a>(
    node: &'a AstNode<'a>, stack: &mut MarkupSpanStack, depth: u32,
) {
    println!(
        "{}{:?}",
        " ".repeat((depth * 2) as usize),
        node.data.borrow()
    );
    node.children()
        .for_each(|child| visit_node(child, stack, depth + 1));
}

// fn visit_title<'a>(
//     node: &'a AstNode<'a>, stack: &mut MarkupSpanStack,
// ) -> Arc<dyn Widget> {
//     let children = node
//         .children()
//         .map(|child| visit_node(child, stack))
//         .collect();
//     Arc::new(Row::new(children))
// }

// fn visit_block_quote(
//     node: &AstNode, stack: &mut MarkupSpanStack,
// ) -> Arc<dyn Widget> {
//     // left bar
//     let left_bar = Arc::new(Container::new());

//     let children = node
//         .children()
//         .map(|child| visit_node(child, stack))
//         .collect();
//     Arc::new(Row::new(children))
// }

// fn visit_paragraph(
//     node: &AstNode, stack: &mut MarkupSpanStack,
// ) -> Arc<dyn Widget> {
//     node.children();
//     // let children = node
//     //     .children()
//     //     .map(|child| visit_node(child, stack))
//     //     .collect();
//     // Arc::new(Row::new(children))
// }
